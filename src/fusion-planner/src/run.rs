use crate::{candidate_grammar::*, session::*};
use serde::{Deserialize, Serialize};
use std::{ffi::CString, time::Instant};
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Request {
    pub prompt: String,
    pub labels: Vec<String>,
    pub candidate_set_digest: String,
    pub mode: String,
    #[serde(default = "retries")]
    pub max_retries: u32,
    #[serde(default = "budget")]
    pub budget_ms: u64,
    #[serde(default)]
    pub ban_all: bool,
}
fn retries() -> u32 {
    8
}
fn budget() -> u64 {
    25000
}
#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct Stats {
    vetoes: u32,
    retries: u32,
    forced_tokens: u32,
    aborted: bool,
}
struct Gate<'a> {
    api: &'a Api,
    labels: &'a [String],
    text: String,
    last: (i32, i32),
    stats: Stats,
    start: Instant,
    budget: u64,
    max: u32,
    ban_all: bool,
    ttft: Option<f64>,
}
extern "C" fn callback(_: *const f32, _: i32, _: i32, data: Handle) -> i32 {
    if data.is_null() {
        return 0;
    }
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        let g = &mut *data.cast::<Gate>();
        if g.start.elapsed().as_millis() >= g.budget as u128 {
            g.stats.aborted = true;
            return 0;
        }
        let id = (g.api.ahem_get_last_sampled_token)();
        if id < 0 {
            return 1;
        }
        let pair = ((g.api.ahem_get_decode_step_index)(), id);
        if g.last == pair {
            return 1;
        }
        let mut buf = [0u8; 512];
        let n = (g.api.ahem_get_last_sampled_token_text)(buf.as_mut_ptr().cast(), 512);
        if n <= 0 || n >= 511 {
            g.stats.aborted = true;
            return 0;
        }
        let piece = String::from_utf8_lossy(&buf[..n as usize]);
        let next = format!("{}{}", g.text, piece);
        let allowed = g
            .labels
            .iter()
            .any(|l| format!("{PREFIX}{l}{SUFFIX}").starts_with(&next));
        let free = g.text == PREFIX;
        if allowed
            && (!free || g.labels.iter().any(|l| l == piece.as_ref()))
            && !(g.ban_all && free)
        {
            g.ttft
                .get_or_insert(g.start.elapsed().as_secs_f64() * 1000.);
            g.text = next;
            g.last = pair;
            (g.api.ahem_clear_banned_tokens)(std::ptr::null_mut());
            (g.api.ahem_set_retry_sampling)(std::ptr::null_mut(), 0);
            if !free {
                g.stats.forced_tokens += 1;
            }
            if free {
                let c = CString::new(SUFFIX).unwrap();
                if (g.api.ahem_append_forced_skeleton)(std::ptr::null_mut(), c.as_ptr()) < 0 {
                    g.stats.aborted = true;
                    return 0;
                }
            }
            if validate(&g.text, g.labels).is_some() {
                (g.api.ahem_set_grammar_complete)();
            }
            1
        } else {
            g.stats.vetoes += 1;
            if g.stats.retries >= g.max {
                g.stats.aborted = true;
                return 0;
            }
            g.stats.retries += 1;
            if (g.api.ahem_ban_token)(std::ptr::null_mut(), id) != 0
                || (g.api.ahem_set_retry_sampling)(std::ptr::null_mut(), g.stats.retries as i32)
                    != 0
            {
                g.stats.aborted = true;
                return 0;
            }
            2
        }
    }))
    .unwrap_or(0)
}
pub fn choose(s: &mut Session, r: Request) -> Result<serde_json::Value, String> {
    if !legal(&r.labels)
        || !matches!(r.mode.as_str(), "ahem-g" | "plain")
        || r.prompt.len() > 8192
        || r.budget_ms == 0
        || r.budget_ms > 25000
        || r.max_retries > 8
    {
        return Err("INVALID_REQUEST".into());
    }
    if s.state != "READY" {
        return Err("NOT_READY".into());
    }
    let replies = r
        .labels
        .iter()
        .map(|l| format!("{PREFIX}{l}{SUFFIX}"))
        .collect::<Vec<_>>()
        .join(" or ");
    let prompt=CString::new(format!("<|im_start|>system\nChoose one of {}. H means hold. Reply only with one of these JSON objects: {}. Do not add other keys or text.<|im_end|>\n<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n",r.labels.join(","),replies,r.prompt)).map_err(|_|"INVALID_REQUEST")?;
    s.api.clear();
    let h = s.handle;
    let tokens = unsafe { planner_token_count(s.api.encode, h, prompt.as_ptr()) };
    if tokens < 0 || tokens > 2048 {
        return Err("INPUT_LIMIT".into());
    }
    unsafe { (s.api.reset)(h) };
    s.state = "GENERATING";
    struct Cleanup<'a>(&'a Api);
    impl Drop for Cleanup<'_> {
        fn drop(&mut self) {
            self.0.clear();
        }
    }
    let _cleanup = Cleanup(&s.api);
    let start = Instant::now();
    let mut g = Gate {
        api: &s.api,
        labels: &r.labels,
        text: String::new(),
        last: (-1, -1),
        stats: Stats::default(),
        start,
        budget: r.budget_ms,
        max: r.max_retries,
        ban_all: r.ban_all,
        ttft: None,
    };
    if r.mode == "ahem-g" {
        unsafe {
            let prefix = CString::new(PREFIX).unwrap();
            if (s.api.ahem_set_forced_skeleton)(h, prefix.as_ptr()) <= 0 {
                return Err("AHEM_UNAVAILABLE".into());
            }
            let literals: Vec<_> = r
                .labels
                .iter()
                .map(|x| CString::new(x.as_str()).unwrap())
                .collect();
            let ptrs: Vec<_> = literals.iter().map(|x| x.as_ptr()).collect();
            (s.api.ahem_set_route_literals)(ptrs.as_ptr(), ptrs.len() as i32);
            (s.api.register_ahem_gate_callback)(callback, (&mut g as *mut Gate).cast());
            if (s.api.ahem_is_gate_callback_registered)() != 1 {
                return Err("AHEM_UNAVAILABLE".into());
            }
        }
    }
    let mut raw = String::new();
    let mut raw_attempts = Vec::new();
    let mut label = None;
    let mut attempts = 0;
    while attempts < if r.mode == "plain" { 2 } else { 1 } {
        attempts += 1;
        if attempts > 1 {
            unsafe { (s.api.reset)(h) };
        }
        let mut buf = [0u8; 8192];
        let n = unsafe {
            (s.api.ahem_llm_response_buf)(h, prompt.as_ptr(), 8, buf.as_mut_ptr().cast(), 8192)
        };
        if n < 0 {
            g.stats.aborted = true;
            break;
        }
        raw = String::from_utf8_lossy(&buf[..n as usize]).into_owned();
        raw_attempts.push(raw.clone());
        label = validate(&raw, &r.labels);
        if label.is_some() {
            break;
        }
    }
    let total = start.elapsed().as_secs_f64() * 1000.;
    if total > r.budget_ms as f64 {
        g.stats.aborted = true;
        s.state = "FAILED";
    } else {
        s.state = "READY";
    }
    if label.is_none() {
        g.stats.aborted = true;
    }
    let output = if g.stats.aborted {
        "H".into()
    } else {
        label.unwrap_or("H".into())
    };
    Ok(
        serde_json::json!({"label":output,"raw":raw,"candidateSetDigest":r.candidate_set_digest,"rawValid":validate(&raw,&r.labels).is_some(),"attempts":attempts,"rawAttempts":raw_attempts,"ahem":{"g":{"mode":r.mode,"vetoes":g.stats.vetoes,"retries":g.stats.retries,"forcedTokens":g.stats.forced_tokens,"aborted":g.stats.aborted},"l":{"mode":"off"}},"timings":{"ttftMs":g.ttft,"totalMs":total}}),
    )
}
