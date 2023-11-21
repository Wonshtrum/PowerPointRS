use powerpointrs::{Slide, Shape, Color, Effect};

fn main() {
    let mut slide = Slide::default();
    let r0 = slide.add(Shape::with_int(1, 1, 1, 1, 1, Color::BLACK));
    let r1 = slide.add(Shape::with_int(2, 2, 2, 2, 2, Color::BLACK));
    let _r = slide.add(Shape::with_int(3, 3, 3, 3, 3, Color::BLACK));
    let r2 = slide.add(Shape::Group {
        z: 3,
        shapes: vec![
            Shape::Group {
                z: 7,
                shapes: vec![
                    Shape::with_int(8, 8, 8, 8, 8, Color::BLACK),
                    Shape::with_int(7, 7, 7, 7, 7, Color::BLACK),
                ],
            },
            Shape::with_int(6, 6, 6, 6, 6, Color::BLACK),
            Shape::with_int(5, 5, 5, 5, 5, Color::BLACK),
        ],
    });
    println!("{r0:?} {r1:?} {r2:?}");
    slide.tl_add(r0, true, Effect::Appear, None);
    slide.tl_add(r1, true, Effect::Appear, Some(r0));
    slide.tl_add(r0, true, Effect::Disappear, Some(r0));
    slide.tl_add(r2, true, Effect::Appear, Some(r1));
    slide.tl_add(r2, true, Effect::Disappear, Some(r2));

    println!("{slide:#?}");
    let mut presentation = slide.presentation();
    println!("{:#?}", presentation.timeline);
    println!("{presentation:#?}");
    presentation.click(1., 1.);
    println!("{presentation:#?}");
    presentation.click(1., 1.);
    println!("{presentation:#?}");
}
