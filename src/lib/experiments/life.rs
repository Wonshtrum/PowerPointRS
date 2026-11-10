use crate::{anim, shape, Referer, Slide};

const W: f32 = 2.0;
const OX: f32 = W;
const OY: f32 = W;
const TX: f32 = 0.0;
const TY: f32 = OY;

struct Cell {
    reset: Referer,
    alive: Referer,
    counter: [Referer; 4],
    connect: [Referer; 8],
}

impl Cell {
    fn new(s: &mut Slide, x: f32, y: f32) -> Self {
        let reset = shape!(@s, x+W/8., y+W/8., W*6./8., W*6./8., z=(-1), c=(255, 255, 200));
        let alive = shape!(@s, x, y, W, W, z=(-1), c=(255, 0, 0));
        let die = shape!(@s, TX, TY, W, W, z=(1), c=(255, 255, 0));

        anim!(@s, alive => Appear, on=reset);
        anim!(@s, alive => Disappear, c=true, on=reset);
        anim!(@s, alive => Place, on=alive);
        anim!(@s, die => Appear, on=alive);
        anim!(@s, alive => Disappear, on=die);
        anim!(@s, die => Disappear, on=die);

        let counter = std::array::from_fn(
            |j| shape!(@s, TX, 0., W, W, z=(0), c=(100 + 30*j as u8, 100 + 30*j as u8, 100 + 30*j as u8)),
        );
        let connect = std::array::from_fn(|_| shape!(@s, TX, TY, W, W, z=(0), c=(0, 255, 255)));

        for (i, counter_shape) in counter.iter().copied().enumerate() {
            if i == 1 {
                anim!(@s, die => Disappear, on=counter_shape);
            }
            if i == 2 {
                anim!(@s, alive => SlideIn, on=counter_shape);
            }
            if i < 3 {
                anim!(@s, counter[i + 1] => SlideIn, on=counter_shape);
            } else {
                anim!(@s, alive => Disappear, on=counter_shape);
                for connector in connect {
                    anim!(@s, connector => Disappear, on=counter_shape);
                }
            }
            anim!(@s, counter_shape => Disappear, on=counter_shape);
        }

        for connector in connect {
            for counter_shape in counter {
                anim!(@s, counter_shape => Target(TX, TY), on=connector);
            }
            anim!(@s, connector => Disappear, on=connector);
        }

        Self {
            reset,
            alive,
            counter,
            connect,
        }
    }
}

pub fn life(rows: usize, cols: usize) -> Slide {
    let mut s = Slide::new((cols + 2) as f32 * W, (rows + 2) as f32 * W);
    let cells: Vec<Vec<Cell>> = (0..rows)
        .map(|j| {
            (0..cols)
                .map(|i| Cell::new(&mut s, OX + i as f32 * W, OY + j as f32 * W))
                .collect()
        })
        .collect();

    for j in 0..rows {
        for i in 0..cols {
            let mut c = 0usize;
            for di in 0..3 {
                for dj in 0..3 {
                    if di == 1 && dj == 1 {
                        continue;
                    }
                    let ni = i as isize + di as isize - 1;
                    let nj = j as isize + dj as isize - 1;
                    if ni >= 0 && nj >= 0 && ni < cols as isize && nj < rows as isize {
                        anim!(@s, cells[nj as usize][ni as usize].connect[7-c] => Appear,
                            on=cells[j][i].alive);
                    }
                    c += 1;
                }
            }
        }
    }

    let pattern: [&str; 9] = [
        "........................O...........",
        "......................O.O...........",
        "............OO......OO............OO",
        "...........O...O....OO............OO",
        "OO........O.....O...OO..............",
        "OO........O...O.OO....O.O...........",
        "..........O.....O.......O...........",
        "...........O...O....................",
        "............OO......................",
    ];
    let generator = shape!(@s, W, 0., W, W, z=(0), c=(0, 0, 255));
    for (j, line) in pattern.iter().enumerate() {
        for (i, ch) in line.chars().enumerate().take(cols) {
            if ch == 'O' && i < cols && j < rows {
                anim!(@s, cells[j][i].alive => Appear, on=generator);
            }
        }
    }

    let start = shape!(@s, TX, TY, W, W, z=(-1), c=(0, 255, 0));
    let step = shape!(@s, TX, TY, W, W, z=(2), c=(255, 0, 255));
    let update = shape!(@s, TX, TY, W, W, z=(1), c=(0, 0, 0), n="UPDATE");

    anim!(@s, start => Disappear, on=start);
    anim!(@s, update => Disappear, on=update);
    anim!(@s, update => Appear, on=step);

    for row in &cells {
        for cell in row {
            anim!(@s, cell.reset => Disappear, on=start);
            for connector in cell.connect {
                anim!(@s, connector => Disappear, on=start);
            }
            anim!(@s, cell.counter[0] => SlideIn, on=step);
            for counter_shape in cell.counter.iter().skip(1).copied() {
                anim!(@s, counter_shape => Disappear, on=step);
            }
            anim!(@s, cell.alive => Target(TX, TY), on=step);
        }
    }
    s
}
