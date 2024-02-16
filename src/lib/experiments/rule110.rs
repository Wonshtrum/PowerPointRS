use crate::{anim, shape, z, Referer, Slide, Z};

struct Cell {
    main: Referer,
    reset: Referer,
    next: Option<Referer>,
}

#[rustfmt::skip]
impl Cell {
    fn new(s: &mut Slide, x: f32, y: f32, w: f32, z: Z, first: bool, last: bool) -> Self {
        let main = shape!(@s,x, y, w, w, Z=z);
        let reset = shape!(@s,x, y - if first { w } else { 0. }, w, w, Z=z, c=(0, 255, 255));
        anim!(@s, main => Appear, on=reset);
        anim!(@s, main => Place, on=main);
        let next = if !last {
            let next = shape!(@s,x, y, w, w, Z=z, c=(255, 0, 0));
            anim!(@s, next => Disappear, on=next);
            Some(next)
        } else {
            None
        };
        Self { main, reset, next }
    }
}

const N_COLUMNS: usize = 10;
const N_ROWS: usize = 10;
const N_GROUP: usize = 3;
const N_M: usize = 1 << N_GROUP;
const W: f32 = 3.;
const W_CELL: f32 = 2.;
const D: f32 = 0.;
const TX: f32 = 0.;
const TY: f32 = 20.;

#[rustfmt::skip]
pub fn rule110() -> Slide {
    let mut s = Slide::new(120., 90.);
    let matrix = (0..N_GROUP)
        .map(|y| y as f32)
        .map(|y| {
            (0..N_GROUP)
                .map(|x| x as f32)
                .map(|x| {
                    let target = shape!(@s,x * (W + 1.),y * (W + 1.), W, W, z=(-2-(N_M as isize)), c=(100*x as u8, 100*y as u8, 0));
                    anim!(@s, target => Disappear, on=target);
                    target
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let stop = shape!(@s,TX, TY, W, W, z=(2+N_ROWS), c=(0, 0, 255), n="STOP");
    let zero = shape!(@s,TX, TY, W, W, z=(-1-(N_M as isize)), c=(120, 120, 120), n="ZERO");
    let start = shape!(@s,TX, TY, W, W, z=(-4-(N_M as isize)), c=(0, 255, 0), n="START");
    let call_in  = (0..N_GROUP).map(|_| shape!(@s,TX, TY, W, W, z=(-3-(N_M as isize)), c=(255, 0, 255))).collect::<Vec<_>>();
    let call_out = (0..N_GROUP).map(|_| shape!(@s,TX, TY, W, W, z=(-(N_M as isize)), c=(255, 200, 255))).collect::<Vec<_>>();
    anim!(@s, start => Disappear, on=start);
    anim!(@s, zero => Disappear, on=start);
    anim!(@s, zero => Disappear, on=zero);
    // anim!(@s, stop => Place, on=stop);
    for target in call_in.iter().chain(call_out.iter()) {
        anim!(@s, *target => Disappear);
        anim!(@s, *target => Disappear, on=*target);
    }
    for index in 0..N_GROUP {
        let i = call_in[index];
        let o = call_out[index];
        anim!(@s, o => Appear, on=i);
        for target in &matrix[index] {
            anim!(@s, *target => Target(TX, TY), on=i);
            anim!(@s, *target => SlideIn, on=o);
        }
    }
    let mut controlers = vec![];
    for i in 0..N_M {
        let s0 = shape!(@s,(N_GROUP+i+1) as f32*(W+1.), 0., W, W, z=(-1-(N_M as isize)), c=(200, 200, 200));
        let s1 = shape!(@s,(N_GROUP+i+1) as f32*(W+1.), 0., W, W, z=(-1-(N_M as isize)), c=(200, 0, 0));
        let s2 = shape!(@s,(N_GROUP+i+1) as f32*(W+1.), W, W, W);
        controlers.push(s0);
        controlers.push(s1);
        anim!(@s, s0 => Disappear, on=s2);
        anim!(@s, s0 => Appear, c=true, on=s2);
        anim!(@s, zero => Appear, on=s0);
        for j in 0..N_GROUP {
            for k in 0..N_GROUP {
                if i & 1 << (N_GROUP-k-1) == 0 {
                    anim!(@s, s0 => Path(TX, TY), on=matrix[j][k]);
                    anim!(@s, s1 => Path(TX, TY), on=matrix[j][k]);
                }
            }
        }
    }
    for c0 in controlers.iter().copied() {
        for c1 in controlers.iter().copied().chain([start]) {
            anim!(@s, c0 => Place, on=c1);
        }
        for c1 in call_in.iter().copied() {
            anim!(@s, c0 => Target(TX, TY), on=c1);
        }
    }
    let ox = s.width - W_CELL;
    let oy = 10.;
    let cells = (0..N_COLUMNS)
        .map(|x| x as f32)
        .map(|x| {
            (0..N_ROWS)
                .map(|y| (y, y as f32))
                .map(|(y, yf)| {
                    Cell::new(&mut s, ox-x*(W_CELL+D), oy+yf*(W_CELL+D), W_CELL, z!(1+N_ROWS-y), y==0, y==N_ROWS-1)
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let mut last = start;
    for y in 0..N_ROWS {
        for target in matrix.iter().flatten().copied() {
            anim!(@s, target => Appear, on=last);
        }
        for x in 0..N_COLUMNS {
            let cell = &cells[x][y];
            anim!(@s, cell.main => Disappear);
            if x > 0 {
                anim!(@s, matrix[x%N_GROUP][0] => Disappear, on=cell.main);
            }
            anim!(@s, matrix[(x+1)%N_GROUP][1] => Disappear, on=cell.main);
            anim!(@s, matrix[(x+2)%N_GROUP][2] => Disappear, on=cell.main);
            if y == 0 {
                anim!(@s, cell.main => Disappear, c=true, on=cell.reset);
            } else {
                anim!(@s, cell.reset => Disappear, on=cell.reset);
                anim!(@s, cell.reset => Disappear, on=zero);
            }
            if let Some(cell_next) = cell.next {
                if x > 0 {
                    anim!(@s, cells[x-1][y+1].reset => Appear, on=cell_next);
                    anim!(@s, cells[x-1][y+1].reset => Target(TX, TY), on=cell_next);
                    anim!(@s, call_in[x%N_GROUP] => Appear, on=cell_next);
                }
                for target in [cell.main, cell_next] {
                    anim!(@s, target => Target(TX, TY), on=last);
                }
                last = cell_next;
            }
        }
        let x = N_COLUMNS-1;
        if y < N_ROWS-1 {
            let tmp = shape!(@s,ox-(x as f32 +1.)*(W_CELL+D), oy+(y as f32)*(W_CELL+D), W_CELL, W_CELL, z=(1+N_ROWS-y), c=(255, 255, 0), n="UPDATE");
            anim!(@s, tmp => Target(TX, TY), on=last);
            anim!(@s, call_in[(x+1)%N_GROUP] => Appear, on=tmp);
            anim!(@s, cells[x][y+1].reset => Target(TX, TY), on=tmp);
            anim!(@s, cells[x][y+1].reset => Appear, on=tmp);
            anim!(@s, tmp => Place, c=true, on=tmp);
            last = tmp;
        }
    }

    s
}
