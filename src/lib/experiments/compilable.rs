use crate::{anim, shape, Slide};

pub fn compilable() -> Slide {
    let mut s = Slide::new(40., 30.);
    let c0 = shape!(@s, 0, 0, 1, 1, c=(255, 0, 0), n="CONTROL_C0");
    let c1 = shape!(@s, 0, 0, 1, 1, c=(255, 0, 0), n="CONTROL_C1");
    let reset = shape!(@s, 0, 0, 1, 1, c=(0, 0, 255), n="RESET");
    let r0 = shape!(@s, 0, 0, 1, 1);
    let r1 = shape!(@s, 0, 0, 2, 1);
    let r2 = shape!(@s, 0, 0, 3, 1);
    let r3 = shape!(@s, 0, 0, 4, 1);
    shape!(@s, 0, 1, 1, 1, c=(0, 255, 255));
    shape!(@s, 1, 1, 1, 1, c=(0, 0, 255));
    shape!(@s, 2, 1, 1, 1, c=(0, 255, 255));
    shape!(@s, 3, 1, 1, 1, c=(0, 0, 255));

    anim!(@s, c0 => Disappear, on=c0);
    anim!(@s, r0 => Disappear, on=c0);
    anim!(@s, r1 => Disappear, on=c0);
    anim!(@s, c1 => Disappear, on=c1);
    anim!(@s, r0 => Disappear, on=c1);
    anim!(@s, r2 => Disappear, on=c1);

    anim!(@s, reset => Disappear);
    anim!(@s, reset => Disappear, on=reset);
    anim!(@s, c0 => Appear, on=reset);
    anim!(@s, c1 => Appear, on=reset);
    anim!(@s, r0 => Appear, on=reset);
    anim!(@s, r1 => Appear, on=reset);
    anim!(@s, r2 => Appear, on=reset);

    anim!(@s, reset => Appear, on=r0);
    anim!(@s, reset => Appear, on=r1);
    anim!(@s, reset => Appear, on=r2);
    anim!(@s, reset => Appear, on=r3);
    s
}
