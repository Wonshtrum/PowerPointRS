use std::{fmt, process::exit};

pub mod experiments;
pub mod render;

//=========================================================
// Shape

#[derive(Clone, Copy)]
pub enum Visibility {
    Hidden,
    Visible,
    Unknown,
}

impl Visibility {
    pub fn is_visible(&self) -> bool {
        matches!(self, Visibility::Unknown | Visibility::Visible)
    }
}

#[derive(Clone)]
#[repr(C)]
pub struct ShapeDynState {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub visibiliy: Visibility,
}

impl ShapeDynState {
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && y >= self.y && x <= self.x + self.w && y <= self.y + self.h
    }

    pub fn is_visible(&self) -> bool {
        self.visibiliy.is_visible()
    }
}

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

#[derive(Clone)]
#[repr(C)]
pub struct ShapeConstState {
    pub color: Color,
    pub x: f32,
    pub y: f32,
}

#[derive(Clone)]
pub struct ShapeState {
    pub state_dyn: ShapeDynState,
    pub state_const: ShapeConstState,
}

impl fmt::Debug for ShapeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "State({}, {}, {}, {})",
            self.state_dyn.x, self.state_dyn.y, self.state_dyn.w, self.state_dyn.h,
        ))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Z(pub isize, pub isize, pub isize);

#[macro_export]
macro_rules! z {
    ($a:expr) => {
        Z($a as isize, 0, 0)
    };
    ($a:expr, $b:expr) => {
        Z($a as isize, $b as isize, 0)
    };
    ($a:expr, $b:expr, $c:expr) => {
        Z($a as isize, $b as isize, $c as isize)
    };
}

#[derive(Clone, Debug)]
pub enum Shape {
    Shape { z: Z, state: ShapeState },
    Group { z: Z, shapes: Vec<Shape> },
}
impl Shape {
    pub fn with_float(x: f32, y: f32, z: Z, w: f32, h: f32, color: Color) -> Shape {
        Shape::Shape {
            z,
            state: ShapeState {
                state_dyn: ShapeDynState {
                    x,
                    y,
                    w,
                    h,
                    visibiliy: Visibility::Unknown,
                },
                state_const: ShapeConstState { color, x, y },
            },
        }
    }
    pub fn with_int(x: i16, y: i16, z: Z, w: i16, h: i16, color: Color) -> Shape {
        Shape::Shape {
            z,
            state: ShapeState {
                state_dyn: ShapeDynState {
                    x: x.into(),
                    y: y.into(),
                    w: w.into(),
                    h: h.into(),
                    visibiliy: Visibility::Unknown,
                },
                state_const: ShapeConstState {
                    color,
                    x: x.into(),
                    y: y.into(),
                },
            },
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

impl Referer {
    pub fn index(&self) -> usize {
        match self {
            Referer::Shape(index) => *index,
            Referer::Group(index, _) => *index,
        }
    }
    pub fn size(&self) -> usize {
        match self {
            Referer::Shape(_) => 0,
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

#[derive(Clone)]
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

    pub fn path(x: f32, y: f32, relative: bool) -> Effect {
        Effect::Path {
            path: Vec::new(),
            x,
            y,
            relative,
        }
    }

    pub fn place() -> Effect {
        Effect::Path {
            path: Vec::new(),
            x: 0.,
            y: 0.,
            relative: true,
        }
    }

    pub fn slide_in(direction: Direction) -> Effect {
        Effect::SlideIn { direction }
    }

    pub fn apply(
        &self,
        state_dyn: &mut ShapeDynState,
        state_const: &ShapeConstState,
    ) -> (bool, bool) {
        let old_visibility = state_dyn.visibiliy;
        match self.preset() {
            Preset::Entr(_, _) => state_dyn.visibiliy = Visibility::Visible,
            Preset::Emph(_, _) => {}
            Preset::Path(_, _) => {}
            Preset::Exit(_, _) => state_dyn.visibiliy = Visibility::Hidden,
        }
        match self {
            Effect::Path { x, y, .. } => {
                state_dyn.x = state_const.x + x;
                state_dyn.y = state_const.y + y;
            }
            Effect::Appear => {}
            Effect::Disappear => {}
            Effect::SlideIn { .. } => {
                state_dyn.x = state_const.x;
                state_dyn.y = state_const.y;
            }
            Effect::SlideOut { .. } => todo!("SlideOut"),
        }
        match (old_visibility, state_dyn.visibiliy) {
            (Visibility::Unknown, Visibility::Hidden)
            | (Visibility::Visible, Visibility::Hidden) => (true, false),
            (_, Visibility::Visible) => (true, true),
            _ => (false, false),
        }
    }

    pub fn init(&mut self, states_dyn: &[ShapeDynState], states_const: &[ShapeConstState]) {
        match self {
            Effect::Path {
                ref mut path,
                ref mut x,
                ref mut y,
                relative,
            } => {
                *path = Vec::new();
                let mut cx = f32::MAX;
                let mut cy = f32::MAX;
                for state in states_const {
                    if state.x < cx {
                        cx = state.x;
                    }
                    if state.y < cy {
                        cy = state.y;
                    }
                }
                if !*relative {
                    *x -= cx;
                    *y -= cy;
                }
            }
            _ => {}
        }
    }
}

#[derive(Clone, Debug)]
pub struct Animation {
    pub target: Referer,
    pub click: bool,
    pub effect: Effect,
}

//=========================================================
// Timeline

#[derive(Clone, Default, Debug)]
pub struct Context {
    pub head: usize,
    pub animations: Vec<Animation>,
}

impl Context {
    fn init(
        &mut self,
        refs: &[Referer],
        shapes_dyn: &mut [ShapeDynState],
        shapes_const: &mut [ShapeConstState],
    ) {
        for animation in &mut self.animations {
            let old_index = animation.target.index();
            let target = refs[old_index];
            animation.target = target;
            let (start, end) = target.bounds();
            animation
                .effect
                .init(&shapes_dyn[start..end], &shapes_const[start..end]);
            for state_dyn in &mut shapes_dyn[start..end] {
                match (animation.effect.preset(), state_dyn.visibiliy) {
                    (Preset::Entr(_, _), Visibility::Unknown) => {
                        state_dyn.visibiliy = Visibility::Hidden
                    }
                    (_, Visibility::Unknown) => state_dyn.visibiliy = Visibility::Visible,
                    _ => {}
                }
            }
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct Timeline {
    pub main_context: Context,
    pub contexts: Vec<Context>,
}

impl Timeline {
    pub fn add(&mut self, target: Referer, click: bool, effect: Effect, on: Option<Referer>) {
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
    pub fn tl_add(&mut self, target: Referer, click: bool, effect: Effect, on: Option<Referer>) {
        self.timeline.add(target, click, effect, on)
    }
    pub fn presentation(mut self) -> Presentation {
        self.shapes.sort_by(|a, b| a.1.z().cmp(&b.1.z()));
        let total_size = self.shapes.iter().map(|e| e.1.size()).sum();
        let mut refs = vec![Referer::Shape(0); self.shapes.len()];
        let mut shapes_dyn = Vec::with_capacity(total_size);
        let mut shapes_const = Vec::with_capacity(total_size);
        let mut shapes_groups = Vec::with_capacity(total_size);
        let mut referer_id = 0;
        for (id, shape) in self.shapes.into_iter().rev() {
            let group_size = shape.size();
            let (mut queue, referer) = match shape {
                Shape::Shape {
                    state:
                        ShapeState {
                            state_dyn,
                            state_const,
                        },
                    ..
                } => {
                    let referer = Referer::Shape(referer_id);
                    refs[id] = referer;
                    shapes_dyn.push(state_dyn);
                    shapes_const.push(state_const);
                    shapes_groups.push(referer);
                    referer_id += 1;
                    continue;
                }
                Shape::Group { mut shapes, .. } => {
                    let referer = Referer::Group(referer_id, group_size);
                    shapes.sort_by(|a, b| b.z().cmp(&a.z()));
                    refs[id] = referer;
                    (shapes, referer)
                }
            };
            while let Some(shape) = queue.pop() {
                match shape {
                    Shape::Shape {
                        state:
                            ShapeState {
                                state_dyn,
                                state_const,
                            },
                        ..
                    } => {
                        shapes_dyn.push(state_dyn);
                        shapes_const.push(state_const);
                        shapes_groups.push(referer);
                        referer_id += 1;
                    }
                    Shape::Group { mut shapes, .. } => {
                        shapes.sort_by(|a, b| b.z().cmp(&a.z()));
                        queue = [queue, shapes].concat();
                    }
                }
            }
        }

        let mut main_context = self.timeline.main_context;
        main_context.init(&refs, &mut shapes_dyn, &mut shapes_const);
        let mut contexts = vec![Context::default(); total_size];
        for (id, mut context) in self.timeline.contexts.into_iter().enumerate() {
            context.init(&refs, &mut shapes_dyn, &mut shapes_const);
            contexts[refs[id].index()] = context;
        }

        Presentation {
            timeline: Timeline {
                main_context,
                contexts,
            },
            cache_hit: CacheHit {
                x: 0.,
                y: 0.,
                index: 0,
            },
            cache_data: CacheData {
                update: false,
                start: 0,
                end: shapes_dyn.len(),
            },
            states_dyn: shapes_dyn,
            states_const: shapes_const,
            referers: shapes_groups,
            width: self.width,
            height: self.height,
        }
    }
}

//=========================================================
// Presentation

#[derive(Clone, Debug)]
pub struct CacheHit {
    pub x: f32,
    pub y: f32,
    pub index: usize,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct CacheData {
    update: bool,
    start: usize,
    end: usize,
}

pub struct Presentation {
    pub states_dyn: Vec<ShapeDynState>,
    pub states_const: Vec<ShapeConstState>,
    pub referers: Vec<Referer>,
    pub timeline: Timeline,
    pub cache_hit: CacheHit,
    pub cache_data: CacheData,
    pub width: f32,
    pub height: f32,
}

impl fmt::Debug for Presentation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "cache_hit: {:?}\ncache_data: {:?}\n",
            self.cache_hit, self.cache_data,
        ))?;
        for state_dyn in &self.states_dyn {
            match state_dyn.visibiliy {
                Visibility::Visible => f.write_str("|=====================")?,
                Visibility::Hidden => f.write_str("|                     ")?,
                Visibility::Unknown => f.write_str("|~~~~~~~~~~~~~~~~~~~~~")?,
            }
        }
        f.write_str("|\n")?;
        for state_dyn in &self.states_dyn {
            f.write_fmt(format_args!(
                "| {:04} {:04} {:04} {:04} ",
                state_dyn.x, state_dyn.y, state_dyn.w, state_dyn.h
            ))?;
        }
        f.write_str("|\n")?;
        for referer in &self.referers {
            match referer {
                Referer::Shape(index) => f.write_fmt(format_args!("| Shape({index:08})     "))?,
                Referer::Group(index, size) => {
                    f.write_fmt(format_args!("| Group({index:08}, {size:02}) "))?
                }
            }
        }
        f.write_str("|\n")?;
        Ok(())
    }
}

impl Presentation {
    pub fn under(&mut self, x: f32, y: f32) -> Option<Referer> {
        for (index, state) in self.states_dyn.iter().enumerate().rev() {
            if state.is_visible() && state.contains(x, y) {
                return Some(self.referers[index]);
            }
        }
        return None;
    }

    pub fn under_cache(&mut self, x: f32, y: f32) -> Option<Referer> {
        let start_index = if self.cache_hit.x == x && self.cache_hit.y == y {
            self.cache_hit.index + 1
        } else {
            self.states_dyn.len()
        };
        for (index, state) in self.states_dyn.iter().enumerate().take(start_index).rev() {
            if state.is_visible() && state.contains(x, y) {
                self.cache_hit = CacheHit { x, y, index };
                return Some(self.referers[index]);
            }
        }
        self.cache_hit = CacheHit { x, y, index: 0 };
        None
    }

    pub fn click(&mut self, x: f32, y: f32) {
        self.cache_data.update = true;
        self.cache_data.start = 0;
        self.cache_data.end = self.states_dyn.len() - 1;
        let (target, context) = match self.under(x, y) {
            Some(referer) => {
                let context = &mut self.timeline.contexts[referer.index()];
                if context.animations.is_empty() {
                    (None, &mut self.timeline.main_context)
                } else {
                    (Some(referer), context)
                }
            }
            None => (None, &mut self.timeline.main_context),
        };
        let mut first = true;
        let head = if context.head == context.animations.len() {
            if target.is_none() {
                exit(1);
            }
            0
        } else {
            context.head
        };
        context.head = context.animations.len();
        for (head, animation) in context.animations.iter_mut().enumerate().skip(head) {
            if !first && animation.click {
                context.head = head;
                break;
            }
            first = false;
            let (start, end) = animation.target.bounds();
            for i in start..end {
                animation
                    .effect
                    .apply(&mut self.states_dyn[i], &self.states_const[i]);
            }
        }
    }

    pub fn click_cache(&mut self, x: f32, y: f32) {
        let (target, context) = match self.under_cache(x, y) {
            Some(referer) => {
                let context = &mut self.timeline.contexts[referer.index()];
                if context.animations.is_empty() {
                    (None, &mut self.timeline.main_context)
                } else {
                    (Some(referer), context)
                }
            }
            None => (None, &mut self.timeline.main_context),
        };
        let mut first = true;
        let head = if context.head == context.animations.len() {
            if target.is_none() {
                exit(1);
            }
            0
        } else {
            context.head
        };
        let mut cache_index = self.cache_hit.index;
        let (mut cache_min, mut cache_max) = if self.cache_data.update {
            (self.cache_data.start, self.cache_data.end)
        } else {
            (self.states_dyn.len(), 0)
        };
        context.head = context.animations.len();
        for (head, animation) in context.animations.iter_mut().enumerate().skip(head) {
            if !first && animation.click {
                context.head = head;
                break;
            }
            first = false;
            let (start, end) = animation.target.bounds();
            for i in start..end {
                let (perceptible, obstructible) = animation
                    .effect
                    .apply(&mut self.states_dyn[i], &self.states_const[i]);
                // let (perceptible, obstructible) = (true, true);
                if obstructible && i > cache_index {
                    cache_index = i;
                }
                if perceptible && i < cache_min {
                    cache_min = i;
                }
                if perceptible && i > cache_max {
                    cache_max = i;
                }
            }
        }
        self.cache_hit.index = cache_index;
        self.cache_data.start = cache_min;
        self.cache_data.end = cache_max;
        self.cache_data.update = true;
    }
}
