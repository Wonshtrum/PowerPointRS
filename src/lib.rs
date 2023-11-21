//=========================================================
// Shape

#[derive(Clone, Copy)]
pub enum Visibility {
    Unknown,
    Hidden,
    Visible,
}

#[derive(Clone)]
pub struct ShapeDynState {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub visibiliy: Visibility,
}

impl ShapeDynState {
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && y >= self.y && x <= self.x + self.w && self.y <= self.y + self.h
    }

    pub fn is_visible(&self) -> bool {
        matches!(self.visibiliy, Visibility::Unknown | Visibility::Visible)
    }
}

#[derive(Clone)]
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
    const fn from_u32(c: u32) -> Self {
        Self {
            r: (c & 0xFF0000 >> 16) as u8,
            g: (c & 0x00FF00 >> 8) as u8,
            b: (c & 0x0000FF >> 0) as u8,
        }
    }
}

#[derive(Clone)]
pub struct ShapeConstState {
    pub color: Color,
}

#[derive(Clone)]
pub struct ShapeState {
    pub state_dyn: ShapeDynState,
    pub state_const: ShapeConstState,
}

impl std::fmt::Debug for ShapeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "State({}, {}, {}, {})",
            self.state_dyn.x, self.state_dyn.y, self.state_dyn.w, self.state_dyn.h,
        ))
    }
}

#[derive(Clone, Debug)]
pub enum Shape {
    Shape { z: usize, state: ShapeState },
    Group { z: usize, shapes: Vec<Shape> },
}
impl Shape {
    pub fn with_float(x: f32, y: f32, z: usize, w: f32, h: f32, color: Color) -> Shape {
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
                state_const: ShapeConstState { color },
            },
        }
    }
    pub fn with_int(x: i16, y: i16, z: usize, w: i16, h: i16, color: Color) -> Shape {
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
                state_const: ShapeConstState { color },
            },
        }
    }
    pub fn z(&self) -> usize {
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

// #[derive(Clone, Copy, Debug)]
// pub enum Direction {
//     Up,
//     Down,
//     Left,
//     Right,
// }

#[derive(Clone, Debug)]
pub enum Effect {
    Appear,
    Disappear,
    SlideIn,
    SlideOut {
        origin: Option<(f32, f32)>,
    },
    Path {
        path: Vec<(f32, f32)>,
        x: f32,
        y: f32,
    },
}

impl Effect {
    pub fn preset(&self) -> Preset {
        match self {
            Effect::Appear => Preset::Entr(1, 0),
            Effect::Disappear => Preset::Exit(1, 0),
            Effect::Path { .. } => Preset::Path(0, 1),
            Effect::SlideIn => Preset::Entr(0, 0),
            Effect::SlideOut { .. } => Preset::Exit(0, 0),
        }
    }

    pub fn place(x: f32, y: f32) -> Effect {
        Effect::Path {
            path: Vec::new(),
            x,
            y,
        }
    }

    pub fn apply(&self, state: &mut ShapeDynState) {
        match self.preset() {
            Preset::Entr(_, _) => state.visibiliy = Visibility::Visible,
            Preset::Emph(_, _) => {}
            Preset::Path(_, _) => {}
            Preset::Exit(_, _) => state.visibiliy = Visibility::Hidden,
        }
        match self {
            Effect::Path { x, y, .. } => {
                state.x = *x;
                state.y = *y;
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
                // let index = animation.referer().index();
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

#[derive(Default, Debug)]
pub struct Slide {
    pub shapes: Vec<(usize, Shape)>,
    pub timeline: Timeline,
}

impl Slide {
    pub fn add(&mut self, shape: Shape) -> Referer {
        let id = self.shapes.len();
        self.shapes.push((id, shape));
        Referer::Shape(id)
    }
    pub fn tl_add(&mut self, target: Referer, click: bool, animation: Effect, on: Option<Referer>) {
        self.timeline.add(target, click, animation, on)
    }
    pub fn presentation(mut self) -> Presentation {
        self.shapes.sort_by(|a, b| a.1.z().cmp(&b.1.z()));
        let total_size = self.shapes.iter().map(|e| e.1.size()).sum();
        let mut refs = vec![Referer::Shape(0); self.shapes.len()];
        let mut shapes_dyn = Vec::with_capacity(total_size);
        let mut shapes_const = Vec::with_capacity(total_size);
        let mut shapes_groups = Vec::with_capacity(total_size);
        let mut referer_id = 0;
        for (id, shape) in self.shapes {
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
        for animation in &mut main_context.animations {
            let old_index = animation.target.index();
            let target = refs[old_index];
            animation.target = target;
            let (start, end) = target.bounds();
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
        let mut contexts = vec![Context::default(); total_size];
        for (id, mut context) in self.timeline.contexts.into_iter().enumerate() {
            for animation in &mut context.animations {
                let old_index = animation.target.index();
                let target = refs[old_index];
                animation.target = target;
                let (start, end) = target.bounds();
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
            contexts[refs[id].index()] = context;
        }

        Presentation {
            states_dyn: shapes_dyn,
            states_const: shapes_const,
            referers: shapes_groups,
            timeline: Timeline {
                main_context,
                contexts,
            },
            cache: CacheHit {
                x: 0.,
                y: 0.,
                index: 0,
            },
        }
    }
}

//=========================================================
// Presentation

pub struct CacheHit {
    pub x: f32,
    pub y: f32,
    pub index: usize,
}

pub struct Presentation {
    pub states_dyn: Vec<ShapeDynState>,
    pub states_const: Vec<ShapeConstState>,
    pub referers: Vec<Referer>,
    pub timeline: Timeline,
    pub cache: CacheHit,
}

impl std::fmt::Debug for Presentation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
        let start_index = if self.cache.x == x && self.cache.y == y {
            self.cache.index
        } else {
            0
        };
        for (index, state) in self.states_dyn.iter().enumerate().skip(start_index) {
            if state.is_visible() && state.contains(x, y) {
                // self.cache = CacheHit { x, y, index };
                return Some(self.referers[index]);
            }
        }
        return None;
    }

    pub fn click(&mut self, x: f32, y: f32) {
        let target = self.under(x, y);
        let context = match target {
            Some(referer) => &mut self.timeline.contexts[referer.index()],
            None => &mut self.timeline.main_context,
        };
        let mut first = true;
        for (head, animation) in &mut context.animations.iter_mut().enumerate().skip(context.head) {
            if !first && animation.click {
                context.head = head;
                return;
            }
            first = false;
            let (start, end) = animation.target.bounds();
            for state in &mut self.states_dyn[start..end] {
                animation.effect.apply(state);
            }
        }
        context.head = 0;
    }
}
