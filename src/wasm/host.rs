#[allow(dead_code)]

pub mod pptrs {
    mod sys {
        #[link(wasm_import_module = "pptrs")]
        extern "C" {
            pub fn log(ptr: *const u8, len: usize);
            pub fn error(ptr: *const u8, len: usize);
        }
    }

    pub fn log(msg: &str) {
        unsafe { sys::log(msg.as_ptr(), msg.len()) }
    }

    pub fn error(msg: &str) {
        unsafe { sys::error(msg.as_ptr(), msg.len()) }
    }
}

#[macro_export]
macro_rules! console {
    ($($t:tt)*) => {
        $crate::host::pptrs::log(&format_args!($($t)*).to_string());
    };
}
