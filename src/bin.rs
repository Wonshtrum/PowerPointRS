use std::time::Instant;

use powerpointrs::{experiments, Color};

fn main() {
    let mut presentation = experiments::rule110();
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
        (118., 8.),
    ];
    for (x, y) in clicks {
        presentation.click(x, y);
    }

    for _ in 0..3000 {
        presentation.click(1., 21.);
    }
    println!("{:?}", start.elapsed());
    println!("{}", presentation.render(1., Color::WHITE).to_string());
}
