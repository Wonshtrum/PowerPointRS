#[cfg(target_arch = "wasm32")]
pub mod wasm {
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

    #[macro_export]
    macro_rules! console {
        ($($t:tt)*) => {
            $crate::host::wasm::log(&format!($($t)*));
        };
    }
    #[macro_export]
    macro_rules! abort {
        ($($t:tt)*) => {
            $crate::host::wasm::error(&format!($($t)*));
        };
    }
}

#[cfg(target_arch = "wasm32")]
pub fn setup_panic() {
    std::panic::set_hook(Box::new(|panic_info| {
        crate::host::wasm::error(&format!("{panic_info}"));
    }));
}
#[cfg(not(target_arch = "wasm32"))]
pub fn setup_panic() {}

#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! console {
    ($($t:tt)*) => {
        println!($($t)*);
    };
}
#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! abort {
    ($($t:tt)*) => {
        panic!($($t)*);
    };
}
