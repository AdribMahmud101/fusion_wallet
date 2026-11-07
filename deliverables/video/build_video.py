#!/usr/bin/env python3
"""Build the Fusion Wallet demo video.

Without voice/full.mp3: a silent cut timed from estimated line lengths.
With voice/full.mp3 (and optional voice/bgm*.mp3): lines are found by silence detection, every shot and subtitle
is timed to the real speech, and the voice is mixed over the music. Subtitles show only while a line is spoken.
"""
import json, os, re, subprocess, sys, textwrap
from PIL import Image, ImageDraw, ImageFont, ImageFilter

HERE = os.path.dirname(os.path.abspath(__file__))
SHOTS, VOICE, OUT = f"{HERE}/shots", f"{HERE}/voice", f"{HERE}/build"
W, H, FPS = 1920, 1080, 30
os.makedirs(OUT, exist_ok=True)

BOLD = "/usr/share/fonts/noto/NotoSans-Bold.ttf"
REG = "/usr/share/fonts/noto/NotoSans-Regular.ttf"
MONO = "/usr/share/fonts/TTF/JetBrainsMonoNL-Medium.ttf"
BG, TEXT, MUTED, GREEN, VIOLET, SKY, RED = (11, 14, 19), (242, 240, 234), (169, 180, 194), (43, 214, 143), (167, 139, 250), (56, 189, 248), (248, 113, 113)

def font(path, size): return ImageFont.truetype(path, size)

def lines_from_script():
    s = open(f"{HERE}/FULL_VOICE_SCRIPT.md").read()
    s = s.split("## Plain version", 1)[1].split("\n", 1)[1]
    return [p.strip() for p in s.split("\n\n") if p.strip() and not p.strip().startswith("(")]

# Visual for each numbered line: ("card", key) or ("clip", shot, start_seconds, label)
PLAN = {
    1: ("card", "blocks"), 2: ("card", "ai"), 3: ("card", "question"),
    4: ("card", "morse"), 5: ("card", "grok"), 6: ("card", "nokey"), 7: ("card", "lesson"),
    8: ("card", "logo"), 9: ("card", "pills"),
    10: ("clip", "s01_home", 0, "Live wallet · Base Sepolia"),
    11: ("clip", "s01_home", 4, "Bills · cushion · spare cash"),
    12: ("clip", "s03_runnow", 0, "One tap: the agent decides"),
    13: ("clip", "s03_runnow", 22, "On-device AI · 6 measured stages"),
    14: ("clip", "s02_earn", 44, "16 policy checks"),
    15: ("card", "hold"),
    16: ("clip", "s02_earn", 34, "75 USDC: actionable"),
    17: ("clip", "s04_autopilot", 0, "Autopilot"),
    18: ("clip", "s05_scout", 0, "Yield Scout · 7 fixed rules"),
    19: ("clip", "s06_why", 1, "Cloud advisor explains"),
    20: ("card", "send"),
    21: ("clip", "s08_safety", 0, "24 / 24 attacks stopped"),
    22: ("card", "asr"),
    23: ("card", "proof"),
    24: ("card", "verify"),
    25: ("card", "listens"), 26: ("card", "rules"), 27: ("card", "keys"),
}

def backdrop(glow=VIOLET, at=(0.8, 0.2)):
    im = Image.new("RGB", (W, H), BG)
    g = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(g)
    cx, cy = int(W * at[0]), int(H * at[1])
    for r, a in ((700, 0.10), (480, 0.16), (300, 0.22)):
        col = tuple(int(BG[i] + (glow[i] - BG[i]) * a) for i in range(3))
        d.ellipse((cx - r, cy - r, cx + r, cy + r), fill=col)
    return Image.blend(im, g.filter(ImageFilter.GaussianBlur(160)), 1.0)

def center_text(d, y, text, f, fill, maxw=1500):
    for line in wrap(text, f, maxw):
        w = d.textlength(line, font=f)
        d.text(((W - w) / 2, y), line, font=f, fill=fill)
        y += int(f.size * 1.25)
    return y

def wrap(text, f, maxw):
    words, out, cur = text.split(), [], ""
    probe = ImageDraw.Draw(Image.new("RGB", (10, 10)))
    for w_ in words:
        t = (cur + " " + w_).strip()
        if probe.textlength(t, font=f) <= maxw: cur = t
        else: out.append(cur); cur = w_
    if cur: out.append(cur)
    return out

def card(key):
    if key in ("blocks", "ai", "question"):
        im = backdrop(SKY, (0.5, 0.35)); d = ImageDraw.Draw(im)
        mono = font(MONO, 30)
        for i in range(12):
            d.text((120 + (i % 4) * 440, 90 + (i // 4) * 60), f"block #{48369529 + i * 7:,}", font=mono, fill=(40, 52, 70))
        big = {"blocks": "Money moves without a pause.", "ai": "And now, an AI is moving it.",
               "question": "What if someone talks the AI into sending it away?"}[key]
        center_text(d, 420, big, font(BOLD, 84), TEXT)
        return im
    if key == "morse":
        im = backdrop(RED, (0.5, 0.4)); d = ImageDraw.Draw(im)
        d.text((160, 150), "MAY 2026 · PUBLIC REPORTS", font=font(BOLD, 30), fill=RED)
        center_text(d, 360, ".. --. -. --- .-. . / .--. .-. . ...- .. --- ..- ... / ... . -. -..", font(MONO, 54), TEXT, 1600)
        center_text(d, 560, "A command hidden in Morse code", font(BOLD, 64), TEXT)
        return im
    if key == "grok":
        im = backdrop(RED, (0.5, 0.4)); d = ImageDraw.Draw(im)
        f = font(BOLD, 72)
        d.rounded_rectangle((250, 330, 760, 520), 28, outline=MUTED, width=3); d.text((340, 385), "Grok", font=f, fill=TEXT)
        d.line((800, 425, 990, 425), fill=RED, width=10); d.polygon([(990, 395), (1030, 425), (990, 455)], fill=RED)
        d.rounded_rectangle((1060, 330, 1670, 520), 28, outline=MUTED, width=3); d.text((1120, 385), "Bankrbot", font=f, fill=TEXT)
        center_text(d, 640, "~$200K in tokens sent (reported)", font(BOLD, 60), RED)
        return im
    if key == "nokey":
        im = backdrop(RED, (0.5, 0.4)); d = ImageDraw.Draw(im)
        y = center_text(d, 330, "No key stolen.", font(BOLD, 84), TEXT)
        y = center_text(d, y + 10, "No contract broken.", font(BOLD, 84), TEXT)
        center_text(d, y + 40, "The AI was asked, and it said yes.", font(BOLD, 60), RED)
        return im
    if key == "lesson":
        im = backdrop(RED, (0.5, 0.4)); d = ImageDraw.Draw(im)
        center_text(d, 360, "If the model can sign anything, anyone who can talk to it can spend your money.", font(BOLD, 72), TEXT, 1500)
        return im
    if key == "logo":
        im = backdrop(VIOLET, (0.5, 0.45)); d = ImageDraw.Draw(im)
        center_text(d, 360, "Fusion Wallet", font(BOLD, 150), TEXT)
        center_text(d, 590, "An AI that runs a treasury on its own, on-chain, and cannot be talked into losing money.", font(REG, 46), MUTED, 1400)
        return im
    if key == "pills":
        im = backdrop(VIOLET, (0.5, 0.45)); d = ImageDraw.Draw(im)
        f = font(BOLD, 56); x = 170
        for txt, col in (("AI acts", GREEN), ("Rules decide", VIOLET), ("Humans hold keys", SKY)):
            w = d.textlength(txt, font=f) + 90
            d.rounded_rectangle((x, 440, x + w, 580), 70, outline=col, width=4); d.text((x + 45, 470), txt, font=f, fill=col); x += w + 60
        return im
    if key == "hold":
        im = backdrop(RED, (0.5, 0.45)); d = ImageDraw.Draw(im)
        d.text((160, 150), "ECONOMICS, NOT VIBES", font=font(BOLD, 30), fill=VIOLET)
        center_text(d, 320, "10 USDC into Aave", font(BOLD, 64), MUTED)
        center_text(d, 430, "57 days", font(BOLD, 170), RED)
        center_text(d, 660, "to pay for its gas, so the AI holds", font(BOLD, 56), TEXT)
        return im
    if key == "asr":
        im = backdrop(GREEN, (0.5, 0.45)); d = ImageDraw.Draw(im)
        center_text(d, 230, "Attacks that succeeded", font(BOLD, 56), MUTED)
        center_text(d, 340, "0 / 30   ·   0 / 20", font(BOLD, 150), GREEN)
        center_text(d, 590, "frozen cases   ·   held-out cases", font(REG, 48), MUTED)
        return im
    if key == "verify":
        im = backdrop(GREEN, (0.5, 0.45)); d = ImageDraw.Draw(im)
        d.rounded_rectangle((180, 260, 1740, 760), 24, fill=(18, 23, 31), outline=(38, 48, 64), width=2)
        m = font(MONO, 40)
        d.text((240, 320), "$ git clone github.com/AdribMahmud101/fusion_wallet", font=m, fill=MUTED)
        d.text((240, 400), "$ python3 scripts/judge/verify_all.py", font=m, fill=TEXT)
        d.text((240, 520), "All 59 verification checks PASSED", font=font(MONO, 48), fill=GREEN)
        d.text((240, 600), "cleanly with 0 keys required.", font=font(MONO, 48), fill=GREEN)
        return im
    if key in ("listens", "rules", "keys"):
        im = backdrop(VIOLET, (0.5, 0.45)); d = ImageDraw.Draw(im)
        big = {"listens": "Who does the AI listen to?", "rules": "Its rules. Its owners. Nothing else.",
               "keys": "The car keys. Never the house keys."}[key]
        center_text(d, 400, big, font(BOLD, 88), TEXT)
        if key == "keys":
            center_text(d, 640, "Fusion Wallet · github.com/AdribMahmud101/fusion_wallet", font(REG, 40), MUTED)
            center_text(d, 700, "Base Sepolia · FinTechathon 2026 · Topic F", font(REG, 36), MUTED)
        return im
    if key == "send":
        im = backdrop(SKY, (0.5, 0.45)); d = ImageDraw.Draw(im)
        d.text((160, 150), "HUMANS HOLD THE KEYS", font=font(BOLD, 30), fill=SKY)
        f = font(BOLD, 42); items = (("You tap Send", TEXT), ("App builds an unsigned Safe tx", MUTED), ("Both owners sign", SKY))
        widths = [d.textlength(t, font=f) + 60 for t, _ in items]; x = (W - sum(widths) - 2 * 36) / 2
        for (txt, col), w in zip(items, widths):
            d.rounded_rectangle((x, 440, x + w, 550), 28, outline=col, width=3); d.text((x + 30, 468), txt, font=f, fill=col); x += w + 36
        center_text(d, 680, "The app holds no owner key. The AI cannot send.", font(BOLD, 52), TEXT)
        return im
    if key == "proof":
        im = backdrop(GREEN, (0.5, 0.45)); d = ImageDraw.Draw(im)
        d.text((160, 150), "PROOF ON-CHAIN", font=font(BOLD, 30), fill=GREEN)
        center_text(d, 300, "41", font(BOLD, 200), GREEN)
        center_text(d, 560, "transactions on Base Sepolia, labelled AI · app · owner · script", font(BOLD, 52), TEXT, 1500)
        center_text(d, 720, "Tap any one in the app: the receipt is checked live", font(REG, 44), MUTED)
        return im
    raise KeyError(key)

PHONE_H = 860
PHONE_W = int(1080 * PHONE_H / 2436) // 2 * 2
PHONE_X, PHONE_Y = 1180, 60

def clip_bg(label):
    im = backdrop(VIOLET, (0.7, 0.3)); d = ImageDraw.Draw(im)
    d.rounded_rectangle((PHONE_X - 14, PHONE_Y - 14, PHONE_X + PHONE_W + 14, PHONE_Y + PHONE_H + 14), 40, fill=(20, 24, 32), outline=(60, 70, 90), width=3)
    d.text((160, 300), "LIVE DEMO", font=font(BOLD, 30), fill=VIOLET)
    y = 360
    for line in wrap(label, font(BOLD, 66), 880):
        d.text((160, y), line, font=font(BOLD, 66), fill=TEXT); y += 84
    return im

def subtitle_png(text, path):
    im = Image.new("RGBA", (W, H), (0, 0, 0, 0)); d = ImageDraw.Draw(im)
    f = font(BOLD, 40); lines = wrap(text, f, 1640)
    lh = 54; top = H - 40 - lh * len(lines) - 24
    d.rounded_rectangle((100, top - 12, W - 100, H - 30), 18, fill=(0, 0, 0, 170))
    y = top
    for ln in lines:
        w = d.textlength(ln, font=f); d.text(((W - w) / 2, y), ln, font=f, fill=(255, 255, 255, 255)); y += lh
    im.save(path)

def run(cmd):
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode: sys.exit(f"ffmpeg failed: {' '.join(cmd)}\n{r.stderr[-1500:]}")

def duration(path):
    return float(subprocess.run(["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", path],
                                capture_output=True, text=True).stdout.strip())

def speech_intervals(audio, n):
    """Speech spans found by silence detection; merge until there are exactly n."""
    r = subprocess.run(["ffmpeg", "-i", audio, "-af", "silencedetect=noise=-35dB:d=0.9", "-f", "null", "-"], capture_output=True, text=True)
    starts = [float(x) for x in re.findall(r"silence_start: ([\d.]+)", r.stderr)]
    ends = [float(x) for x in re.findall(r"silence_end: ([\d.]+)", r.stderr)]
    total = duration(audio)
    sil = list(zip(starts, ends + [total] * (len(starts) - len(ends))))
    spans, cur = [], 0.0
    for s, e in sil:
        if s - cur > 0.15: spans.append([cur, s])
        cur = e
    if total - cur > 0.15: spans.append([cur, total])
    while len(spans) > n:  # merge the pair with the shortest gap (a breath inside one line)
        i = min(range(len(spans) - 1), key=lambda k: spans[k + 1][0] - spans[k][1])
        spans[i][1] = spans[i + 1][1]; del spans[i + 1]
    if len(spans) != n: sys.exit(f"found {len(spans)} spoken lines in {audio}, expected {n}; adjust the pauses")
    return spans, total

def main():
    lines = lines_from_script(); n = len(lines)
    audio = next((f"{VOICE}/full.{e}" for e in ("mp3", "wav", "m4a") if os.path.exists(f"{VOICE}/full.{e}")), None)
    if audio:
        spans, total = speech_intervals(audio, n)
        lead = spans[0][0]
        seg = [(spans[i][0], (spans[i + 1][0] if i + 1 < n else total) - spans[i][0], spans[i][1] - spans[i][0]) for i in range(n)]
    else:
        gaps = {3: 2.5, 7: 3.0, 9: 2.0, 20: 2.0, 22: 2.0, 24: 2.5}
        t, seg, lead = 3.0, [], 3.0
        for i, ln in enumerate(lines, 1):
            sd = len(ln.split()) / 2.3 + 0.3; g = gaps.get(i, 1.5) if i < n else 3.0
            seg.append((t, sd + g, sd)); t += sd + g
        total = t
    json.dump({"lines": lines, "segments": seg, "lead": lead}, open(f"{OUT}/timing.json", "w"), indent=1)
    parts = []
    intro = f"{OUT}/seg00.mp4"; card("blocks").save(f"{OUT}/intro.png")
    run(["ffmpeg", "-y", "-loop", "1", "-t", f"{lead:.3f}", "-i", f"{OUT}/intro.png", "-vf", f"fade=in:0:{FPS},fps={FPS},format=yuv420p", "-c:v", "libx264", "-preset", "veryfast", "-crf", "20", intro])
    parts.append(intro)
    for i, (start, dur, speak) in enumerate(seg, 1):
        out, sub = f"{OUT}/seg{i:02d}.mp4", f"{OUT}/sub{i:02d}.png"
        subtitle_png(lines[i - 1], sub)
        kind = PLAN[i]
        en = f"enable='lt(t,{speak:.3f})'"
        if kind[0] == "card":
            card(kind[1]).save(f"{OUT}/bg{i:02d}.png")
            run(["ffmpeg", "-y", "-loop", "1", "-t", f"{dur:.3f}", "-i", f"{OUT}/bg{i:02d}.png", "-loop", "1", "-t", f"{dur:.3f}", "-i", sub,
                 "-filter_complex", f"[0:v][1:v]overlay=0:0:{en},fps={FPS},format=yuv420p", "-c:v", "libx264", "-preset", "veryfast", "-crf", "20", out])
        else:
            _, shot, s0, label = kind
            src = f"{SHOTS}/{shot}.mp4"
            s0 = min(s0, max(0.0, duration(src) - 1.0))
            clip_bg(label).save(f"{OUT}/bg{i:02d}.png")
            run(["ffmpeg", "-y", "-loop", "1", "-t", f"{dur:.3f}", "-i", f"{OUT}/bg{i:02d}.png", "-ss", f"{s0:.2f}", "-i", src, "-loop", "1", "-t", f"{dur:.3f}", "-i", sub,
                 "-filter_complex", f"[1:v]scale={PHONE_W}:{PHONE_H},fps={FPS},tpad=stop_mode=clone:stop_duration={dur:.3f}[p];"
                                    f"[0:v][p]overlay={PHONE_X}:{PHONE_Y}:shortest=1[v];[v][2:v]overlay=0:0:{en},format=yuv420p",
                 "-t", f"{dur:.3f}", "-an", "-c:v", "libx264", "-preset", "veryfast", "-crf", "20", out])
        parts.append(out)
        print(f"line {i:2d} {dur:5.1f}s {kind[0]}", flush=True)
    open(f"{OUT}/concat.txt", "w").write("".join(f"file '{p}'\n" for p in parts))
    silent = f"{HERE}/Fusion_Wallet_Demo_silent.mp4"
    ins = [x for p in parts for x in ("-i", p)]
    norm = "".join(f"[{k}:v]fps={FPS},settb=1/{FPS},setpts=PTS-STARTPTS,format=yuv420p[v{k}];" for k in range(len(parts)))
    run(["ffmpeg", "-y", *ins, "-filter_complex", norm + "".join(f"[v{k}]" for k in range(len(parts))) + f"concat=n={len(parts)}:v=1:a=0[v]",
         "-map", "[v]", "-c:v", "libx264", "-preset", "veryfast", "-crf", "20", silent])
    print("silent:", silent, f"{duration(silent):.1f}s")
    if audio:
        bgms = sorted(f"{VOICE}/{f}" for f in os.listdir(VOICE) if f.startswith("bgm") and f.endswith((".mp3", ".wav", ".m4a")))
        final = f"{HERE}/Fusion_Wallet_Demo.mp4"
        if bgms:
            ins, fc = [], ""
            for k, b in enumerate(bgms): ins += ["-i", b]
            if len(bgms) == 1: fc = "[2:a]volume=0.22,afade=t=out:st=%.2f:d=3[m];" % (total - 3)
            else: fc = "[2:a][3:a]acrossfade=d=3[mm];[mm]volume=0.22,afade=t=out:st=%.2f:d=3[m];" % (total - 3)
            fc += "[1:a]volume=1.0[vo];[m][vo]sidechaincompress=threshold=0.05:ratio=6:attack=20:release=400[duck];[duck][vo]amix=inputs=2:duration=longest:normalize=0[a]"
            run(["ffmpeg", "-y", "-i", silent, "-i", audio, *ins, "-filter_complex", fc.replace("[1:a]volume=1.0[vo];", "[1:a]asplit=2[vo][vo2];").replace("[duck][vo]", "[duck][vo2]"),
                 "-map", "0:v", "-map", "[a]", "-c:v", "copy", "-c:a", "aac", "-b:a", "192k", "-shortest", final])
        else:
            run(["ffmpeg", "-y", "-i", silent, "-i", audio, "-map", "0:v", "-map", "1:a", "-c:v", "copy", "-c:a", "aac", "-b:a", "192k", "-shortest", final])
        print("final:", final, f"{duration(final):.1f}s")

if __name__ == "__main__":
    main()
