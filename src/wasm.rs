pub mod pptrs {
    // use crate::*;

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

// #[no_mangle]
// pub extern "C" fn init_panic_hook() {
//     core::panic::set_hook(Box::new(|info| {
//         let msg = info.to_string();
//         pptrs::error(&msg);
//     }));
//     pptrs::log("Panic Hook successfully initialized");
// }
