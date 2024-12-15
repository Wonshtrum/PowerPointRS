use std::{fmt, io::BufRead};

pub mod bitvec;
pub mod experiments;
pub mod filters;
pub mod render;
pub mod runners;

//=========================================================
// Debug

pub fn pause() {
    let _ = std::io::stdin().lock().read_line(&mut String::new());
}

//=========================================================
// Shape

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const WHITE: Self = Self::from_u32(0xFFFFFF);
    pub const BLACK: Self = Self::from_u32(0x000000);
    pub const RED: Self = Self::from_u32(0xFF0000);
    pub const GREEN: Self = Self::from_u32(0x00FF00);
    pub const BLUE: Self = Self::from_u32(0x0000FF);
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    pub const fn from_u32(c: u32) -> Self {
        Self {
            r: ((c & 0xFF0000) >> 16) as u8,
            g: ((c & 0x00FF00) >> 8) as u8,
            b: ((c & 0x0000FF) >> 0) as u8,
        }
    }
    pub const fn grey(c: u8) -> Self {
        Self { r: c, g: c, b: c }
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(rgb: (u8, u8, u8)) -> Self {
        Self::new(rgb.0, rgb.1, rgb.2)
    }
}

#[derive(Clone)]
pub struct ShapeState {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: Color,
}

impl fmt::Debug for ShapeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "State({}, {}, {}, {})",
            self.x, self.y, self.w, self.h,
        ))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Z(pub isize, pub isize, pub isize);

#[derive(Clone, Debug)]
pub enum Shape {
    Shape {
        z: Z,
        name: &'static str,
        state: ShapeState,
    },
    Group {
        z: Z,
        shapes: Vec<Shape>,
    },
}
impl Shape {
    pub fn new(x: f32, y: f32, w: f32, h: f32, z: Z, color: Color, name: &'static str) -> Shape {
        Shape::Shape {
            z,
            name,
            state: ShapeState { x, y, w, h, color },
        }
    }
    pub fn z(&self) -> Z {
        match self {
            Shape::Shape { z, .. } => *z,
            Shape::Group { z, .. } => *z,
        }
    }
    pub fn size(&self) -> usize {
        match self {
            Shape::Shape { .. } => 1,
            Shape::Group { shapes, .. } => shapes.iter().map(Shape::size).sum(),
        }
    }
}

#[macro_export]
macro_rules! z {
    ($a:expr) => {
        $crate::Z($a as isize, 0, 0)
    };
    ($a:expr, $b:expr) => {
        $crate::Z($a as isize, $b as isize, 0)
    };
    ($a:expr, $b:expr, $c:expr) => {
        $crate::Z($a as isize, $b as isize, $c as isize)
    };
}

#[macro_export]
macro_rules! shape {
    (@$s:expr, $($t:tt)*) => {
        $s.add(shape!{ $($t)* })
    };
    ($x:expr, $y:expr, $w:expr, $h:expr $(,Z=$Z:expr)? $(,z=($($z:expr),*))? $(,c=$c:expr)? $(,n=$n:expr)? $(,)?) => {{
           let _z = $crate::Z(0,0,0);
        $( let _z = $Z; )?
        $( let _z = $crate::z!($($z),*); )?
           let _c = $crate::Color::BLACK;
        $( let _c = $c.into(); )?
           let _n = "";
        $( let _n = $n; )?
        $crate::Shape::Shape {
            z: _z,
            name: _n,
            state: $crate::ShapeState {
                x: $x as f32,
                y: $y as f32,
                w: $w as f32,
                h: $h as f32,
                color: _c,
            },
        }
    }};
}

// pub enum ShapeIterator {
//     End,
//     Single(ShapeState),
//     Stack(Vec<(Vec<Shape>, usize)>),
// }

// impl Iterator for ShapeIterator {
//     type Item = ShapeState;

//     pub fn next(&mut self) -> Option<Self::Item> {
//             let iter = std::mem::replace(self, ShapeIterator::End);
//             match iter {
//                 ShapeIterator::Single (state) => {*self = ShapeIterator::End; Some(state)},
//                 ShapeIterator::Stack(mut stack) => {
//                     match stack.last() {
//                         None | Some((_, 0)) => {*self=ShapeIterator::End; None},
//                         Some((shapes, i)) => {
//                             // *self=ShapeIterator::Stack(stack);
//                             match shapes[*i] {
//                                 Shape::Shape { z, state } => todo!(),
//                                 Shape::Group { z, shapes } => todo!(),
//                             }
//                         },
//                     }
//                 }
//                 ShapeIterator::End => None,
//             }

//     }
// }

//=========================================================
// Animation

#[derive(Clone, Copy, Debug)]
pub enum Referer {
    Shape(usize),
    Group(usize, usize),
}

impl std::default::Default for Referer {
    fn default() -> Self {
        Self::Shape(0)
    }
}

impl Referer {
    pub fn index(&self) -> usize {
        match self {
            Referer::Shape(index) => *index,
            Referer::Group(index, _) => *index,
        }
    }
    pub fn size(&self) -> usize {
        match self {
            Referer::Shape(_) => 1,
            Referer::Group(_, size) => *size,
        }
    }

    pub fn span(&self) -> (usize, usize) {
        match self {
            Referer::Shape(index) => (*index, 1),
            Referer::Group(index, size) => (*index, *size),
        }
    }
    pub fn bounds(&self) -> (usize, usize) {
        match self {
            Referer::Shape(index) => (*index, *index + 1),
            Referer::Group(index, size) => (*index, *index + *size),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Preset {
    Entr(u8, u8),
    Emph(u8, u8),
    Path(u8, u8),
    Exit(u8, u8),
}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Up,
    Down,
    Right,
    Left,
}

#[derive(Clone, Debug)]
pub enum Effect {
    Appear,
    Disappear,
    SlideIn {
        direction: Direction,
    },
    SlideOut {
        direction: Direction,
        origin: Option<(f32, f32)>,
        complete: bool,
    },
    Path {
        path: Vec<(f32, f32)>,
        x: f32,
        y: f32,
        relative: bool,
    },
}

impl Effect {
    pub fn preset(&self) -> Preset {
        match self {
            Effect::Appear => Preset::Entr(1, 0),
            Effect::Disappear => Preset::Exit(1, 0),
            Effect::Path { .. } => Preset::Path(0, 1),
            Effect::SlideIn { .. } => Preset::Entr(0, 0),
            Effect::SlideOut { .. } => Preset::Exit(0, 0),
        }
    }
}

pub enum MacroEffect {
    Appear,
    Disappear,
    SlideIn,
    SlideOut,
    Mark,
    Place,
    Target(f32, f32),
    Path(f32, f32),
}

impl From<MacroEffect> for Effect {
    fn from(effect: MacroEffect) -> Self {
        match effect {
            MacroEffect::Appear => Self::Appear,
            MacroEffect::Disappear => Self::Disappear,
            MacroEffect::SlideIn => Self::SlideIn {
                direction: Direction::Up,
            },
            MacroEffect::SlideOut => Self::SlideOut {
                direction: Direction::Up,
                origin: None,
                complete: true,
            },
            MacroEffect::Mark => Self::SlideOut {
                direction: Direction::Up,
                origin: None,
                complete: false,
            },
            MacroEffect::Place => Self::Path {
                path: Vec::new(),
                x: 0.,
                y: 0.,
                relative: true,
            },
            MacroEffect::Target(x, y) => Self::Path {
                path: Vec::new(),
                x,
                y,
                relative: false,
            },
            MacroEffect::Path(x, y) => Self::Path {
                path: Vec::new(),
                x,
                y,
                relative: true,
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct Animation {
    pub target: Referer,
    pub click: bool,
    pub effect: Effect,
}

#[macro_export]
macro_rules! anim {
    (@$s:expr, $t:expr => $e:tt$(($($args:tt)+))? $(, c=$c:expr)? $(, on=$on:expr)?) => {
        anim!(@$s, $t => $crate::MacroEffect::$e$(($($args)+))? $(, c=$c)? $(, on=$on)?)
    };

    (@$s:expr, $t:expr => $e:expr $(, c=$c:expr)? $(, on=$on:expr)?) => {{
           let _c = false;
        $( let _c = $c; )?
           let _on = Option::<$crate::Referer>::None;
        $( let _on = Some($on); )?
        $s.tl_add($t, $e.into(), _c, _on)
    }};
}

//=========================================================
// Timeline

#[derive(Clone, Default, Debug)]
pub struct Context {
    pub head: usize,
    pub animations: Vec<Animation>,
}

#[derive(Clone, Default, Debug)]
pub struct Timeline {
    pub main_context: Context,
    pub contexts: Vec<Context>,
}

impl Timeline {
    pub fn add(&mut self, target: Referer, effect: Effect, click: bool, on: Option<Referer>) {
        let animation = Animation {
            target,
            click,
            effect,
        };
        match on {
            Some(referer) => {
                let index = referer.index();
                let length = self.contexts.len();
                for _ in length..(index + 1) {
                    self.contexts.push(Context::default())
                }
                self.contexts[index].animations.push(animation);
            }
            None => self.main_context.animations.push(animation),
        }
    }
}

//=========================================================
// Slide

#[derive(Debug)]
pub struct Slide {
    pub shapes: Vec<(usize, Shape)>,
    pub timeline: Timeline,
    pub width: f32,
    pub height: f32,
}

impl Slide {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            shapes: Vec::new(),
            timeline: Timeline::default(),
            width,
            height,
        }
    }
    pub fn add(&mut self, shape: Shape) -> Referer {
        let id = self.shapes.len();
        self.shapes.push((id, shape));
        Referer::Shape(id)
    }
    pub fn tl_add(&mut self, target: Referer, effect: Effect, click: bool, on: Option<Referer>) {
        self.timeline.add(target, effect, click, on)
    }
}
