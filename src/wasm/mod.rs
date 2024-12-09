#![no_main]

use powerpointrs::{experiments, runners::basic::Presentation, Color};
mod host;

#[no_mangle]
pub extern "C" fn main() -> Box<Presentation> {
    // let slide = experiments::test();
    // let slide = experiments::sort();
    let slide = experiments::rule110();
    Box::new(Presentation::from(slide))
}

#[no_mangle]
pub extern "C" fn display(presentation: &Presentation) {
    console!("{presentation:#?}");
}
#[no_mangle]
pub extern "C" fn print(presentation: &Presentation, scale: f32) {
    console!("{}", presentation.render(scale, Color::WHITE));
}

#[no_mangle]
pub extern "C" fn click(presentation: &mut Presentation, x: f32, y: f32, n: usize) {
    for _ in 0..n {
        presentation.click(x, y);
    }
}

const DYNAMIC: usize = 0;
const CONSTANT: usize = 1;

#[no_mangle]
pub extern "C" fn get_vbo_ptr(presentation: &mut Presentation, index: usize) -> *mut u8 {
    match index {
        DYNAMIC => presentation.states_dyn.as_mut_ptr() as _,
        CONSTANT => presentation.states_const.as_mut_ptr() as _,
        _ => 0 as _,
    }
}
#[no_mangle]
pub extern "C" fn get_vbo_size(presentation: &mut Presentation, index: usize) -> usize {
    match index {
        DYNAMIC => presentation.states_dyn.len(),
        CONSTANT => presentation.states_const.len(),
        _ => 0,
    }
}

#[no_mangle]
pub extern "C" fn get_subdata_slice(presentation: &mut Presentation) -> *const u8 {
    &presentation.cache_data as *const _ as _
}

#[no_mangle]
pub extern "C" fn get_width(presentation: &mut Presentation) -> f32 {
    presentation.width
}
#[no_mangle]
pub extern "C" fn get_height(presentation: &mut Presentation) -> f32 {
    presentation.height
}
