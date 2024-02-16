use std::collections::HashMap;

use crate::{z, Color, Effect, Referer, Shape, Slide, Z};

#[repr(usize)]
enum Instruction {
    INC = 0,
    DEC = 1,
    LEFT = 2,
    RIGHT = 3,
    PUSH = 4,
    POP = 5,
    OUT = 6,
    IN = 7,
}

const N_CELLS: usize = 65;
const N_WORDS: usize = 10;
const N_CHARS: usize = 15;
const N_BITS: usize = 6;
const N_STACK: usize = 3 + 1;

const W: f32 = 1.5;
const TX: f32 = 0.;
const TY: f32 = 50.;

fn target() -> Effect {
    Effect::path(TX, TY, false)
}

#[derive(Default, Clone, Copy, Debug)]
struct Bit {
    not_zero: Referer,
    zero: Referer,
    one: Referer,
    next0: Option<Referer>,
    next1: Option<Referer>,
}

const BLACK: Color = Color::BLACK;
impl Bit {
    fn new(s: &mut Slide, x: f32, y: f32, w: f32, first: bool) -> Self {
        let not_zero = s.add(Shape::with_float(
            0.,
            y,
            z!(-4),
            w / 2.0,
            w,
            Color::new(255, 0, 255),
        ));
        let zero = s.add(Shape::with_float(x, y, z!(0), w, w, BLACK)); // 0
        let one = s.add(Shape::with_float(x, y, z!(0), w, w, BLACK)); // 1

        s.tl_add(not_zero, false, Effect::Appear, Some(zero));
        s.tl_add(one, false, Effect::Appear, Some(zero));
        s.tl_add(zero, false, Effect::Disappear, Some(zero));

        s.tl_add(not_zero, false, Effect::Disappear, Some(one));
        s.tl_add(zero, false, Effect::Appear, Some(one));
        s.tl_add(one, false, Effect::Disappear, Some(one));

        let (next0, next1) = if first {
            (None, None)
        } else {
            let next0 = s.add(Shape::with_float(x, y, z!(1), w, w, BLACK));
            let next1 = s.add(Shape::with_float(x, y, z!(1), w, w, BLACK));

            s.tl_add(next1, false, Effect::path(TX, TY, false), Some(one));
            s.tl_add(next0, false, target(), Some(zero));
            s.tl_add(next1, false, Effect::place(), Some(next1));
            s.tl_add(next0, false, Effect::place(), Some(next0));

            (Some(next0), Some(next1))
        };

        Self {
            not_zero,
            zero,
            one,
            next0,
            next1,
        }
    }

    fn link(&self, s: &mut Slide, other: &Bit) {
        s.tl_add(other.zero, false, target(), self.next0);
        s.tl_add(other.one, false, target(), self.next0);
        s.tl_add(other.zero, false, target(), self.next1);
        s.tl_add(other.one, false, target(), self.next1);
    }
}

struct Word {
    bits: [Bit; N_BITS],
    inc: Referer,
    dec: Referer,
    left: Referer,
    right: Referer,
    reset: Referer,
    test: Referer,
    set: Referer,
}

fn t(val: f32, cond: bool) -> f32 {
    if cond {
        val
    } else {
        0.
    }
}

impl Word {
    fn new(s: &mut Slide, x: f32, y: f32, w: f32, h: bool) -> Self {
        let v = !h;
        let mut bits = [Bit::default(); N_BITS];

        for i in 0..N_BITS {
            let bit = Bit::new(s, x + t(w * i as f32, h), y + t(w * i as f32, v), w, false);
            if i > 0 {
                bit.link(s, &bits[i - 1]);
            }
            bits[i] = bit;
        }

        let i = N_BITS as f32 - 1.0;
        let inc = s.add(Shape::with_float(
            x + t((i + 1.0) * w, h),
            y + t((i + 1.0) * w, v),
            z!(-1),
            w,
            w,
            Color::new(0, 255, 0),
        )); // I
        let dec = s.add(Shape::with_float(
            x + t((i + 2.0) * w, h),
            y + t((i + 2.0) * w, v),
            z!(-1),
            w,
            w,
            Color::new(255, 0, 0),
        )); // D
        let left = s.add(Shape::with_float(
            x + t((i + 3.0) * w, h),
            y + t((i + 3.0) * w, v),
            z!(-1),
            w,
            w,
            Color::new(0, 0, 255),
        )); // ⯇
        let right = s.add(Shape::with_float(
            x + t((i + 3.0) * w, h),
            y + t((i + 3.0) * w, v),
            z!(-1),
            w,
            w,
            Color::new(0, 0, 255),
        )); // ⯈
        let reset = s.add(Shape::with_float(
            x + t((i + 4.0) * w, h),
            y + t((i + 4.0) * w, v),
            z!(-1),
            w,
            w,
            Color::new(120, 120, 120),
        )); // R
        let test = s.add(Shape::with_float(
            x + t((i + 5.0) * w, h),
            y + t((i + 5.0) * w, v),
            z!(-1),
            w,
            w,
            Color::new(255, 0, 255),
        )); // T
        let set = s.add(Shape::with_float(
            x + t((i + 6.0) * w, h),
            y + t((i + 6.0) * w, v),
            z!(-1),
            w,
            w,
            Color::new(255, 255, 0),
        )); // I

        s.tl_add(reset, false, Effect::place(), Some(reset));

        for control in [set, inc, dec, left, right, test] {
            s.tl_add(control, false, Effect::place(), Some(left));
            s.tl_add(control, false, Effect::place(), Some(right));
        }

        for (i, bit) in bits.iter().enumerate() {
            if let (Some(next0), Some(next1)) = (bit.next0, bit.next1) {
                s.tl_add(next0, false, Effect::place(), Some(reset));
                s.tl_add(next1, false, Effect::place(), Some(reset));
                s.tl_add(next1, false, Effect::Appear, Some(inc));
                s.tl_add(next0, false, Effect::Appear, Some(dec));
                s.tl_add(next0, false, Effect::Disappear, Some(inc));
                s.tl_add(next1, false, Effect::Disappear, Some(dec));
            }
            s.tl_add(bit.not_zero, false, target(), Some(test));

            for target in [set, bit.zero, bit.one] {
                s.tl_add(
                    bit.zero,
                    false,
                    Effect::path(TX, TY + (i as f32 - N_BITS as f32) * w, false),
                    Some(target),
                );
                s.tl_add(
                    bit.one,
                    false,
                    Effect::path(TX, TY + (i as f32 - N_BITS as f32) * w, false),
                    Some(target),
                );
            }
        }

        let lsb = bits.last().unwrap();
        s.tl_add(lsb.one, false, target(), Some(inc));
        s.tl_add(lsb.one, false, target(), Some(dec));
        s.tl_add(lsb.zero, false, target(), Some(inc));
        s.tl_add(lsb.zero, false, target(), Some(dec));
        s.tl_add(reset, false, target(), Some(inc));
        s.tl_add(reset, false, target(), Some(dec));

        Self {
            bits,
            inc,
            dec,
            left,
            right,
            reset,
            test,
            set,
        }
    }

    fn get_controls(&self) -> Vec<Referer> {
        vec![
            self.set, self.inc, self.dec, self.left, self.right, self.test,
        ]
    }
}

struct Cell {
    symbols: Vec<Referer>,
    stack: Vec<(Option<Referer>, Option<Referer>)>,
    cycle: Referer,
    place: Referer,
    next: Referer,
    skip: Referer,
}

impl Cell {
    const SYMBOLS: &'static str = "+-<>[].,";

    fn new(s: &mut Slide, x: f32, y: f32, w: f32) -> Self {
        let cycle = s.add(Shape::with_float(x, y + w, z!(0), w, w, BLACK));
        let place = s.add(Shape::with_float(
            x,
            y + w * 2.0,
            z!(2),
            w,
            w,
            Color::new(0, 0, 255),
        )); // _UPDATE
        let next = s.add(Shape::with_float(
            x,
            y + w * 3.0,
            z!(0),
            w,
            w,
            Color::new(255, 0, 255),
        ));
        let skip = s.add(Shape::with_float(
            x,
            y + w * 4.0,
            z!(-1),
            w,
            w,
            Color::new(255, 128, 0),
        )); // [

        let symbols: Vec<Referer> = Cell::SYMBOLS
            .chars()
            .map(|symbol| {
                s.add(Shape::with_float(
                    x,
                    y,
                    z!(if "[]".contains(symbol) { -4 } else { 0 }),
                    w,
                    w,
                    BLACK,
                )) // symbol
            })
            .collect();

        let stack = (0..N_STACK)
            .map(|i| {
                let up = if i == N_STACK - 1 {
                    None
                } else {
                    Some(s.add(Shape::with_float(
                        x,
                        y - w * (i as f32 + 1.0),
                        z!(0),
                        w / 2.0,
                        w,
                        Color::new(255, 0, 255),
                    )))
                };
                let down = if i == 0 {
                    None
                } else {
                    Some(s.add(Shape::with_float(
                        x + w / 2.0,
                        y - w * (i as f32 + 1.0),
                        z!(0),
                        w / 2.0,
                        w,
                        Color::new(255, 0, 255),
                    )))
                };
                (up, down)
            })
            .collect::<Vec<_>>();

        for (i, symbol) in symbols.iter().copied().enumerate() {
            s.tl_add(symbol, true, Effect::Appear, Some(cycle));
            s.tl_add(symbol, false, target(), Some(place));
            if i > 0 {
                s.tl_add(symbols[i - 1], false, Effect::Disappear, Some(cycle));
            }
        }

        for (i, (up, down)) in stack.iter().copied().enumerate() {
            if let (Some(up), Some((next_up, Some(next_down)))) = (up, stack.get(i + 1).copied()) {
                if let Some(next_up) = next_up {
                    s.tl_add(next_up, false, Effect::Appear, Some(up));
                    s.tl_add(next_up, false, Effect::place(), Some(up));
                }
                s.tl_add(next_down, false, Effect::Appear, Some(up));
                s.tl_add(next_down, false, Effect::place(), Some(up));
                s.tl_add(up, false, Effect::Disappear, Some(up));
                if let Some(down) = down {
                    s.tl_add(down, false, Effect::Disappear, Some(up));
                }
            }
            if let (Some(down), Some((Some(prev_up), prev_down))) =
                (down, stack.get(i - 1).copied())
            {
                s.tl_add(prev_up, false, Effect::Appear, Some(down));
                if let Some(prev_down) = prev_down {
                    s.tl_add(prev_down, false, Effect::Appear, Some(down));
                    s.tl_add(prev_down, false, Effect::place(), Some(down));
                }
                s.tl_add(prev_up, false, Effect::place(), Some(down));
                if let Some(up) = up {
                    s.tl_add(up, false, Effect::Disappear, Some(down));
                }
                s.tl_add(down, false, Effect::Disappear, Some(down));
            }
        }

        s.tl_add(
            stack[0].0.unwrap(),
            false,
            target(),
            Some(symbols[Instruction::PUSH as usize]),
        );
        s.tl_add(
            skip,
            false,
            target(),
            Some(symbols[Instruction::PUSH as usize]),
        );
        s.tl_add(stack[1].1.unwrap(), false, target(), Some(next));
        s.tl_add(place, false, target(), Some(next));
        s.tl_add(skip, false, Effect::Disappear, Some(next));

        s.tl_add(place, false, Effect::place(), Some(place));
        s.tl_add(next, false, Effect::place(), Some(next));
        s.tl_add(skip, false, Effect::place(), Some(skip));

        Self {
            symbols,
            stack,
            cycle,
            place,
            next,
            skip,
        }
    }
}

struct Character {
    symbols: Vec<Referer>,
    save: Referer,
    reveal: Referer,
}

impl Character {
    const SYMBOLS: &'static str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

    fn new(s: &mut Slide, x: f32, y: f32, w: f32, controler: &[Referer]) -> Self {
        let symbols = Character::SYMBOLS
            .chars()
            .map(|symbol| s.add(Shape::with_float(x, y, z!(0), w, w, BLACK)))
            .collect::<Vec<_>>();

        let save = s.add(Shape::with_float(
            x,
            y + w,
            z!(0),
            w,
            w,
            Color::new(0, 255, 0),
        ));
        let reveal = s.add(Shape::with_float(
            TX,
            TY,
            z!(3),
            w,
            w,
            Color::new(128, 128, 128),
        ));

        for (i, symbol) in symbols.iter().copied().enumerate() {
            s.tl_add(symbol, false, Effect::mark(), Some(save));
            for (j, bit) in controler.iter().copied().enumerate() {
                if i & (1 << j) == 0 {
                    s.tl_add(symbol, false, Effect::path(0., -w, true), Some(bit));
                }
            }
            s.tl_add(symbol, false, Effect::path(0., 0., false), Some(reveal));
        }

        s.tl_add(save, false, Effect::Appear, Some(reveal));
        s.tl_add(save, false, target(), Some(reveal));
        s.tl_add(reveal, false, Effect::Disappear, Some(reveal));

        Self {
            symbols,
            save,
            reveal,
        }
    }
}

pub fn brainfck() {
    let mut s = Slide::new(120., 90.);
    Cell::new(&mut s, 0., 0., W);
}

fn bfck() {
    let mut s = Slide::new(120., 90.);
    let mut ox = 10.0;
    let mut oy = TX;
    let w = W;

    // Create cells
    let cells: Vec<Cell> = (0..N_CELLS)
        .map(|x| Cell::new(&mut s, ox + x as f32 * w, oy, w))
        .collect();

    // Reset coordinates
    ox = 10.0;
    oy = 10.0;

    // Create words
    let words: Vec<Word> = (0..N_WORDS)
        .map(|i| Word::new(&mut s, ox + w * i as f32, oy, w, false))
        .collect();

    // Reset coordinates
    ox = 50.0;
    oy = 10.0;

    // Create character controller
    let char_controler = (0..N_BITS)
        .map(|i| s.add(Shape::with_float(ox + w * i as f32, oy + 4.0 * w, z!(-4), w, w, Color::new(255, 255, 0))))
        .collect::<Vec<_>>();

    // Create characters
    let chars = (0..N_CHARS)
        .map(|i| Character::new(&mut s, ox + w * i as f32 * 2.0, oy, w * 2.0, &char_controler))
        .collect::<Vec<_>>();

    // Initialize controler object
    let ctl_symbols = HashMap::new();

    // Create controler symbols
    for (i, symbol) in Cell::SYMBOLS.chars().enumerate() {
        ctl_symbols.insert(i, s.add(Shape::with_float(ox + i as f32 * w, oy, z!(-3), w, w, Color::new(255, 0, 0))));
    }

    // Add additional controler shapes
    let ctl_enter = s.add(Shape::with_float(TX, TY - (N_BITS + 1) * w, z!(1), w, w, Color::new(255, 0, 0))); // E
    let ctl_reset = s.add(Shape::with_float(ox + (i + 1) * w + 1., oy, z!(-3), w, w, Color::new(255, 0, 0))); // R
    let ctl_loop  = s.add(Shape::with_float(ox + (i + 2) * w + 2., oy, z!(-3), w, w, Color::new(255, 0, 0))); // L
    let ctl_skip  = s.add(Shape::with_float(ox + (i + 3) * w + 2., oy, z!(-3), w, w, Color::new(255, 128, 0))); // S
    let ctl_cover = s.add(Shape::with_float(ox + (i + 4) * w + 2., oy, z!(-3), w, w, Color::new(255, 128, 0))); // C
    let ctl_reset_cover = s.add(Shape::with_float(ox + (i + 5) * w + 2., oy, z!(-3), w, w, Color::new(255, 128, 0))); // R

    // s.add(Shape::with_float(ox + (i + j + 5.5) * w + 3., oy, z!(-3), w/2., w/2., Color::new(255, 128, 0))); // R

    let ctl_covers = (0..N_STACK)
    .rev()
    .map(|i| {
        let up = if i == N_STACK - 1 { None } else { Some(s.add(
            Shape::with_float(ox + (i + j + 5.5) * w + 3., oy, z!(-3), w/2., w, Color::new(255, 128, 0))
        ))};
        let down = if i == 0 { None } else { Some(s.add(
            Shape::with_float(ox + (i + j + 5.) * w + 3., oy, z!(-3), w/2., w, Color::new(255, 128, 0))
        ))};
        (up, down)
    })
    .rev()
    .collect::<Vec<_>>();

    for (i, (up, down)) in ctl_covers.iter().enumerate() {

    }

}
