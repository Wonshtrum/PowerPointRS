use crate::{anim, shape, z, Shape, Slide};

pub fn test() -> Slide {
    let mut slide = Slide::new(40., 30.);
    let r0 = shape!(@slide, 1, 1, 1, 1, z=(1), c=(255, 0, 0));
    let r1 = shape!(@slide, 2, 2, 2, 2, z=(2), c=(0, 255, 0));
    let r2 = shape!(@slide, 3, 3, 3, 3, z=(3), c=(0, 0, 255));
    let r3 = slide.add(Shape::Group {
        z: z!(3),
        shapes: vec![
            Shape::Group {
                z: z!(7),
                shapes: vec![
                    shape! {8, 8, 8, 8, z=(8), c=(250, 200, 0)},
                    shape! {7, 7, 7, 7, z=(7), c=(200, 250, 0)},
                ],
            },
            shape! {6, 6, 6, 6, z=(6), c=(250, 0, 200)},
            shape! {5, 5, 5, 5, z=(5), c=(200, 0, 250)},
        ],
    });
    println!("{r0:?} {r1:?} {r2:?}");
    anim!(@slide, r0 => Appear);
    anim!(@slide, r1 => Appear, on=r0);
    anim!(@slide, r0 => Disappear, on=r0);
    anim!(@slide, r2 => Target(1., 1.), on=r2);
    anim!(@slide, r2 => Target(1., 5.), on=r2);
    anim!(@slide, r2 => Place, on=r2);
    anim!(@slide, r3 => Appear, on=r1);
    anim!(@slide, r3 => Target(1., 1.), on=r3);
    anim!(@slide, r3 => Disappear, on=r3);
    anim!(@slide, r3 => Target(0., 0.), on=r3);

    slide
}
