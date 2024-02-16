use crate::{shape, z, Effect, Shape, Slide, Z};

pub fn compilable1() -> Slide {
    let mut slide = Slide::new(40., 30.);
    let r0 = shape!(@slide, 1, 1, 4, 4, c=(255, 0, 0), n="red");
    let r1 = shape!(@slide, 2, 2, 4, 4, c=(0, 255, 0), n="green");
    // slide.tl_add(target, click, effect, on)
    slide
}
