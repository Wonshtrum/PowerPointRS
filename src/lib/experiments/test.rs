use crate::{z, Color, Effect, Shape, Slide, Z};

pub fn test() -> Slide {
    let mut slide = Slide::new(40., 30.);
    let r0 = slide.add(Shape::with_int(1, 1, z!(1), 1, 1, Color::new(255, 0, 0)));
    let r1 = slide.add(Shape::with_int(2, 2, z!(2), 2, 2, Color::new(0, 255, 0)));
    let r2 = slide.add(Shape::with_int(3, 3, z!(3), 3, 3, Color::new(0, 0, 255)));
    let r3 = slide.add(Shape::Group {
        z: z!(3),
        shapes: vec![
            Shape::Group {
                z: z!(7),
                shapes: vec![
                    Shape::with_int(8, 8, z!(8), 8, 8, Color::new(250, 200, 0)),
                    Shape::with_int(7, 7, z!(7), 7, 7, Color::new(200, 250, 0)),
                ],
            },
            Shape::with_int(6, 6, z!(6), 6, 6, Color::new(250, 0, 200)),
            Shape::with_int(5, 5, z!(5), 5, 5, Color::new(200, 0, 250)),
        ],
    });
    println!("{r0:?} {r1:?} {r2:?}");
    slide.tl_add(r0, true, Effect::Appear, None);
    slide.tl_add(r1, true, Effect::Appear, Some(r0));
    slide.tl_add(r0, true, Effect::Disappear, Some(r0));
    slide.tl_add(r2, true, Effect::path(1., 1., false), Some(r2));
    slide.tl_add(r2, true, Effect::path(1., 5., false), Some(r2));
    slide.tl_add(r2, true, Effect::place(), Some(r2));
    slide.tl_add(r3, true, Effect::Appear, Some(r1));
    slide.tl_add(r3, true, Effect::path(1., 1., true), Some(r3));
    slide.tl_add(r3, true, Effect::Disappear, Some(r3));
    slide.tl_add(r3, true, Effect::path(0., 0., false), Some(r3));

    slide
}
