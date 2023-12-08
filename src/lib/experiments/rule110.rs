use crate::{z, Color, Direction, Effect, Presentation, Referer, Shape, Slide, Z};

struct Cell {
    main: Referer,
    reset: Referer,
    next: Option<Referer>,
}

#[rustfmt::skip]
impl Cell {
    fn new(s: &mut Slide, x: f32, y: f32, w: f32, z: Z, first: bool, last: bool) -> Self {
        let main = s.add(Shape::with_float(x, y, z, w, w, BLACK));
        let reset = s.add(Shape::with_float(x, y - if first { w } else { 0. }, z, w, w, Color::new(0, 255, 255)));
        s.tl_add(main, false, Effect::Appear, Some(reset));
        s.tl_add(main, false, Effect::place(), Some(main));
        let next = if !last {
            let next = s.add(Shape::with_float(x, y, z, w, w, Color::new(255, 0, 0)));
            s.tl_add(next, false, Effect::Disappear, Some(next));
            Some(next)
        } else {
            None
        };
        Self { main, reset, next }
    }
}

const N_COLUMNS: usize = 20;
const N_ROWS: usize = 20;
const N_GROUP: usize = 3;
const N_M: usize = 1 << N_GROUP;
const W: f32 = 3.;
const D: f32 = 0.;
const TX: f32 = 0.;
const TY: f32 = 20.;
const BLACK: Color = Color::BLACK;

#[rustfmt::skip]
pub fn rule110() -> Box<Presentation> {
    let mut s = Slide::new(120., 90.);
    let matrix = (0..N_GROUP)
        .into_iter()
        .map(|y| y as f32)
        .map(|y| {
            (0..N_GROUP)
                .into_iter()
                .map(|x| x as f32)
                .map(|x| {
                    let target = s.add(Shape::with_float(x * (W + 1.), y * (W + 1.), z!(-2-(N_M as isize)), W, W, Color::new(100*x as u8, 100*y as u8, 0)));
                    s.tl_add(target, false, Effect::Disappear, Some(target));
                    target
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let stop = s.add(Shape::with_float(TX, TY, z!(2+N_ROWS), W, W, Color::new(0, 0, 255)));
    let zero = s.add(Shape::with_float(TX, TY, z!(-1-(N_M as isize)), W, W, Color::grey(120)));
    let start = s.add(Shape::with_float(TX, TY, z!(-4-(N_M as isize)), W, W, Color::new(0, 255, 0))); //_UPDATE
    let call_in  = (0..N_GROUP).into_iter().map(|_|
        s.add(Shape::with_float(TX, TY, z!(-3-(N_M as isize)), W, W, Color::new(255, 0, 255)))
    ).collect::<Vec<_>>();
    let call_out = (0..N_GROUP).into_iter().map(|_|
        s.add(Shape::with_float(TX, TY, z!(-(N_M as isize)), W, W, Color::new(255, 200, 255)))
    ).collect::<Vec<_>>();
    s.tl_add(start, false, Effect::Disappear, Some(start));
    s.tl_add(zero, false, Effect::Disappear, Some(start));
    s.tl_add(zero, false, Effect::Disappear, Some(zero));
    s.tl_add(stop, false, Effect::place(), Some(stop));
    for target in call_in.iter().chain(call_out.iter()) {
        s.tl_add(*target, false, Effect::Disappear, None);
        s.tl_add(*target, false, Effect::Disappear, Some(*target));
    }
    for index in 0..N_GROUP {
        let i = call_in[index];
        let o = call_out[index];
        s.tl_add(o, false, Effect::Appear, Some(i));
        for target in &matrix[index] {
            s.tl_add(*target, false, Effect::path(TX, TY, false), Some(i));
            s.tl_add(*target, false, Effect::slide_in(Direction::Down), Some(o));
        }
    }
    let mut controlers = vec![];
    for i in 0..N_M {
        let s0 = s.add(Shape::with_float((N_GROUP+i+1) as f32*(W+1.), 0., z!(-1-(N_M as isize)), W, W, Color::grey(200)));
        let s1 = s.add(Shape::with_float((N_GROUP+i+1) as f32*(W+1.), 0., z!(-1-(N_M as isize)), W, W, Color::new(200, 0, 0)));
        let s2 = s.add(Shape::with_float((N_GROUP+i+1) as f32 *(W+1.), W, z!(0), W, W, BLACK));
        controlers.push(s0);
        controlers.push(s1);
        s.tl_add(s0, false, Effect::Disappear, Some(s2));
        s.tl_add(s0, true, Effect::Appear, Some(s2));
        s.tl_add(zero, false, Effect::Appear, Some(s0));
        for j in 0..N_GROUP {
            for k in 0..N_GROUP {
                if i & 1 << (N_GROUP-k-1) == 0 {
                    s.tl_add(s0, false, Effect::path(TX, TY, true), Some(matrix[j][k]));
                    s.tl_add(s1, false, Effect::path(TX, TY, true), Some(matrix[j][k]));
                }
            }
        }
    }
    for c0 in &controlers {
        for c1 in controlers.iter().chain(&[start]) {
            s.tl_add(*c0, false, Effect::place(), Some(*c1));
        }
        for c1 in &call_in {
            s.tl_add(*c0, false, Effect::path(TX, TY, false), Some(*c1));
        }
    }
    let ox = s.width - W;
    let oy = 10.;
    let cells = (0..N_COLUMNS)
        .into_iter()
        .map(|x| x as f32)
        .map(|x| {
            (0..N_ROWS)
                .into_iter()
                .map(|y| (y, y as f32))
                .map(|(y, yf)| {
                    Cell::new(&mut s, ox-x*(W+D), oy+yf*(W+D), W, z!(1+N_ROWS-y), y==0, y==N_ROWS-1)
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    
    let mut last = start;
    for y in 0..N_ROWS {
        for target in matrix.iter().flat_map(|v| v).map(|r| *r) {
            s.tl_add(target, false, Effect::Appear, Some(last));
        }
        for x in 0..N_COLUMNS {
            let cell = &cells[x][y];
            s.tl_add(cell.main, false, Effect::Disappear, None);
            if x > 0 {
                s.tl_add(matrix[x%N_GROUP][0], false, Effect::Disappear, Some(cell.main));
            }
            s.tl_add(matrix[(x+1)%N_GROUP][1], false, Effect::Disappear, Some(cell.main));
            s.tl_add(matrix[(x+2)%N_GROUP][2], false, Effect::Disappear, Some(cell.main));
            if y == 0 {
                s.tl_add(cell.main, true, Effect::Disappear, Some(cell.reset));
            } else {
                s.tl_add(cell.reset, false, Effect::Disappear, Some(cell.reset));
                s.tl_add(cell.reset, false, Effect::Disappear, Some(zero));
            }
            if let Some(cell_next) = cell.next {
                if x > 0 {
                    s.tl_add(cells[x-1][y+1].reset, false, Effect::Appear, Some(cell_next));
                    s.tl_add(cells[x-1][y+1].reset, false, Effect::path(TX, TY, false), Some(cell_next));
                    s.tl_add(call_in[x%N_GROUP], false, Effect::Appear, Some(cell_next));
                }
                for target in [cell.main, cell_next] {
                    s.tl_add(target, false, Effect::path(TX, TY, false), Some(last));
                }
                last = cell_next;
            }
        }
        let x = N_COLUMNS-1;
        let i = N_GROUP-1;
        if y < N_ROWS-1 {
            let tmp = s.add(Shape::with_float(ox-(x as f32 +1.)*(W+D), oy+(y as f32)*(W+D), z!(1+N_ROWS-y), W, W, Color::new(255, 255, 0))); //_UPDATE
            s.tl_add(tmp, false, Effect::path(TX, TY, false), Some(last));
            s.tl_add(call_in[(x+i)%N_GROUP], false, Effect::Appear, Some(tmp));
            s.tl_add(cells[x][y+1].reset, false, Effect::path(TX, TY, false), Some(tmp));
            s.tl_add(cells[x][y+1].reset, false, Effect::Appear, Some(tmp));
            s.tl_add(tmp, true, Effect::place(), Some(tmp));
            last = tmp;
        }
    }
    let presentation = s.presentation();
    Box::new(presentation)
}
