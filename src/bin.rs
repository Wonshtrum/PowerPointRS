use std::time::Instant;

use powerpointrs::{
    experiments,
    runners::{
        basic::Presentation as BasicPresentation, minimal::Presentation as MinimalPresentation,
    },
    Color,
};

fn main() {
    let slide = experiments::rule110();
    let mut presentation = MinimalPresentation::from(slide, (0., 20.));
    // let mut presentation = BasicPresentation::from(slide);
    let start = Instant::now();

    let clicks = [
        // trigger main
        (15., 15.),
        // set rule 110 (0b01101110)
        (22., 4.),
        (26., 4.),
        (34., 4.),
        (38., 4.),
        (42., 4.),
        // set initial state
        (120., 10.),
    ];
    for (x, y) in clicks {
        presentation.click(x, y);
    }

    // presentation.update_filter(0., 20.);
    for _ in 0..26500 {
        presentation.click(0., 20.);
    }
    println!("{:?}", start.elapsed());
    println!("{}", presentation.render(1., Color::WHITE).to_string());
    println!("iterations: {}", presentation.iters);
}
