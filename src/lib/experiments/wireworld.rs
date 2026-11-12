use crate::{anim, shape, Referer, Slide};

const W: f32 = 2.0;
const OX: f32 = W;
const OY: f32 = W;
const TX: f32 = 0.0;
const TY: f32 = OY;

const DEBUG: f32 = 0.0;

struct Cell {
    base: Referer,
    wire: Referer,
    tail: Referer,
    head: Referer,
    counter: [Referer; 3],
    connect: [Referer; 8],
    confirm: Referer,
    delete: Referer,
}

impl Cell {
    fn new(s: &mut Slide, x: f32, y: f32) -> Self {
        let base = shape!(@s, x, y, W, W, z=(5));
        let wire = shape!(@s, x, y, W, W, z=(4), c=(255, 200, 0));
        let tail = shape!(@s, x, y, W, W, z=(3), c=(255, 0, 0));
        let head = shape!(@s, x, y, W, W, z=(-1), c=(0, 0, 255));

        let mw = W / 3.0;
        let counter = std::array::from_fn(|i| {
            let debug_x = DEBUG * (x + 12.0 * W);
            let debug_y = DEBUG * (y + (i as f32 + 3.0) * mw);
            shape!(@s, debug_x, debug_y, mw * 3.0, mw, z=(0), c=(100, 100, 100))
        });
        const DIRECTIONS: [(usize, usize); 8] = [
            (0, 0),
            (0, 1),
            (0, 2),
            (1, 0),
            (1, 2),
            (2, 0),
            (2, 1),
            (2, 2),
        ];
        let connect = std::array::from_fn(|k| {
            let (i, j) = DIRECTIONS[k];
            let debug_x = DEBUG * (x + i as f32 * mw + 6.0 * W);
            let debug_y = DEBUG * (y + j as f32 * mw);
            shape!(@s, debug_x, debug_y, mw, mw, z=(0), c=(0, 255, 0))
        });

        let confirm = shape!(@s, TX, TY, W, W, z=(2), c=(0, 255, 255));
        let delete = shape!(@s, x, y, W, W, z=(0), c=(255, 100, 0), n="HIDE");

        for i in 0..3 {
            if i < 2 {
                anim!(@s, counter[i + 1] => SlideIn, on=counter[i]);
            }
            anim!(@s, counter[i] => Disappear, on=counter[i]);
        }

        let reset = counter[2];
        for connector in connect {
            anim!(@s, connector => SlideIn, on=connector);
            anim!(@s, confirm => SlideIn, on=connector);
            for counter_shape in counter {
                anim!(@s, counter_shape => Target(TX, TY), on=connector);
            }
            anim!(@s, connector => SlideIn, on=reset);
        }

        anim!(@s, confirm => Disappear);
        anim!(@s, confirm => Disappear, on=reset);
        anim!(@s, confirm => Disappear, on=confirm);
        anim!(@s, head => SlideIn, on=confirm);

        Self {
            base,
            wire,
            tail,
            head,
            counter,
            connect,
            confirm,
            delete,
        }
    }

    fn on_head(&self, s: &mut Slide) {
        let base = self.base;
        let wire = self.wire;
        let tail = self.tail;
        let head = self.head;
        let delete = self.delete;
        let confirm = self.confirm;

        for counter_shape in self.counter {
            anim!(@s, counter_shape => Disappear, on=head);
        }

        for connector in self.connect {
            anim!(@s, connector => SlideIn, on=tail);
            anim!(@s, connector => SlideIn, on=base);
            anim!(@s, connector => Disappear, on=wire);
            anim!(@s, connector => Disappear, on=confirm);
        }

        anim!(@s, delete => Appear, on=head);
        anim!(@s, delete => Disappear, on=delete);
        anim!(@s, tail => Disappear, on=delete);

        anim!(@s, wire => Appear, on=base);
        anim!(@s, head => Appear, on=wire);
        anim!(@s, tail => SlideIn, on=head);
        anim!(@s, wire => Appear, on=tail);
        anim!(@s, wire => Disappear, on=wire);
        anim!(@s, tail => Disappear, on=tail);
        anim!(@s, head => Disappear, on=head);
    }
}

pub fn wireworld(rows: usize, cols: usize) -> Slide {
    let mut s = Slide::new((cols + 2) as f32 * W, (rows + 2) as f32 * W);

    let cells: Vec<Vec<Cell>> = (0..rows)
        .map(|j| {
            (0..cols)
                .map(|i| Cell::new(&mut s, OX + i as f32 * W, OY + j as f32 * W))
                .collect()
        })
        .collect();

    for i in 0..cols {
        for j in 0..rows {
            let cell = &cells[j][i];
            let mut c = 0usize;
            for di in 0..3 {
                for dj in 0..3 {
                    if di == 1 && dj == 1 {
                        continue;
                    }
                    let ni = i as isize + di as isize - 1;
                    let nj = j as isize + dj as isize - 1;
                    if ni >= 0 && nj >= 0 && ni < cols as isize && nj < rows as isize {
                        let neighbour = &cells[nj as usize][ni as usize];
                        anim!(@s, neighbour.connect[c] => Target(TX, TY), on=cell.head);
                        cell.on_head(&mut s);
                    }
                    c += 1;
                }
            }
        }
    }

    let cover = shape!(@s, OX, OY, cols as f32 * W, rows as f32 * W, c=(255, 0, 0));
    let start = shape!(@s, TX, TY, W, W, z=(-1), c=(0, 255, 0));
    let update = shape!(@s, TX, TY, W, W, z=(6), c=(255, 0, 255), n="UPDATE");
    let step = shape!(@s, TX, TY, W, W, z=(-1), c=(255, 0, 255));

    anim!(@s, cover => Disappear);
    anim!(@s, step => Appear, on=update);
    anim!(@s, step => Disappear, on=step);
    anim!(@s, start => Disappear, on=start);

    for i in 0..cols {
        for j in 0..rows {
            let cell = &cells[j][i];
            anim!(@s, cell.head => Target(TX, TY), on=step);
            anim!(@s, cell.tail => Target(TX, TY), on=step);
            anim!(@s, cell.counter[0] => SlideIn, on=step);
            anim!(@s, cell.delete => Target(0., 0.), on=start);
            for connector in cell.connect {
                anim!(@s, connector => Place, on=start);
            }
        }
    }

    s
}
