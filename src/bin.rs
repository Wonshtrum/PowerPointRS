use std::time::Instant;

use powerpointrs::experiments;

fn main() {
    let mut presentation = experiments::rule110();
    let start = Instant::now();
    for i in 0..10000 {
        presentation.click_cache(0.9, 21.);
        // println!("{i}, {:?}", presentation.cache_hit);
    }
    println!("{:?}", start.elapsed());
}
