use std::ffi::{CString, c_char, c_void};
pub type Handle = *mut c_void;
pub type Callback = extern "C" fn(*const f32, i32, i32, Handle) -> i32;
macro_rules! api { ($($name:ident: $ty:ty),* $(,)?) => {
 pub struct Api { _lib:libloading::Library, pub reset:unsafe extern "C" fn(Handle), pub encode:*mut c_void, $(pub $name:$ty),* }
 impl Api { pub fn open(path:&std::path::Path)->Result<Self,String> { unsafe {
 let lib=libloading::Library::new(path).map_err(|_|"AHEM_UNAVAILABLE")?;
 $(let $name= *lib.get::<$ty>(concat!(stringify!($name),"\0").as_bytes()).map_err(|_|"AHEM_UNAVAILABLE")?;)*
 let reset=*lib.get::<unsafe extern "C" fn(Handle)>(b"_ZN3MNN11Transformer3Llm5resetEv\0").map_err(|_|"AHEM_UNAVAILABLE")?;
 let encode=*lib.get::<*mut c_void>(b"_ZN3MNN11Transformer3Llm16tokenizer_encodeERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE\0").map_err(|_|"AHEM_UNAVAILABLE")?;
 Ok(Self{_lib:lib,reset,encode,$($name),*})
 } } }
}; }
api! {
 register_ahem_gate_callback:unsafe extern "C" fn(Callback,Handle),
 ahem_llm_create:unsafe extern "C" fn(*const c_char)->Handle,
 ahem_llm_destroy:unsafe extern "C" fn(Handle),
 ahem_llm_response_buf:unsafe extern "C" fn(Handle,*const c_char,i32,*mut c_char,i32)->i32,
 clear_ahem_gate_callback:unsafe extern "C" fn(),
 ahem_is_gate_callback_registered:unsafe extern "C" fn()->i32,
 ahem_get_last_sampled_token:unsafe extern "C" fn()->i32,
 ahem_get_decode_step_index:unsafe extern "C" fn()->i32,
 ahem_get_last_sampled_token_text:unsafe extern "C" fn(*mut c_char,i32)->i32,
 ahem_ban_token:unsafe extern "C" fn(Handle,i32)->i32,
 ahem_clear_banned_tokens:unsafe extern "C" fn(Handle),
 ahem_set_retry_sampling:unsafe extern "C" fn(Handle,i32)->i32,
 ahem_set_forced_skeleton:unsafe extern "C" fn(Handle,*const c_char)->i32,
 ahem_append_forced_skeleton:unsafe extern "C" fn(Handle,*const c_char)->i32,
 ahem_clear_forced_skeleton:unsafe extern "C" fn(),
 ahem_set_grammar_complete:unsafe extern "C" fn(),
 ahem_is_grammar_complete:unsafe extern "C" fn()->i32,
 ahem_set_route_literals:unsafe extern "C" fn(*const *const c_char,i32)->i32,
}
impl Api {
    pub fn clear(&self) {
        unsafe {
            (self.clear_ahem_gate_callback)();
            (self.ahem_clear_forced_skeleton)();
            (self.ahem_clear_banned_tokens)(std::ptr::null_mut());
            (self.ahem_set_retry_sampling)(std::ptr::null_mut(), 0);
        }
    }
}
pub struct Session {
    pub api: Api,
    pub config: String,
    pub state: &'static str,
    pub handle: Handle,
}
impl Session {
    pub fn create(&self) -> Result<Handle, String> {
        let c = CString::new(self.config.as_str()).map_err(|_| "LOAD_FAILED")?;
        let h = unsafe { (self.api.ahem_llm_create)(c.as_ptr()) };
        if h.is_null() {
            Err("LOAD_FAILED".into())
        } else {
            Ok(h)
        }
    }
}

// All access to Session and the process-global gate is serialized by SESSION.
unsafe impl Send for Session {}
impl Drop for Session {
    fn drop(&mut self) {
        self.api.clear();
        if !self.handle.is_null() {
            unsafe { (self.api.ahem_llm_destroy)(self.handle) }
        }
    }
}

unsafe extern "C" {
    pub fn planner_token_count(encode: *mut c_void, handle: Handle, text: *const c_char) -> i32;
}
