use std::{collections::HashMap, fmt, process::exit};

use crate::{
    filters::{Cell, DoubleFilter, CELL_MASK, CELL_SHIFT},
    render::Canvas,
    Color, Context, Effect, Shape, ShapeState, Slide,
};

#[derive(Clone)]
#[repr(C)]
pub struct ShapeConstState {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: Color,
}

#[derive(Clone, Default)]
pub struct CellOp {
    pub set: Cell,
    pub unset: Cell,
}

impl std::fmt::Debug for CellOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CellOp")
            .field("set", &format!("{:064b}", self.set))
            .field("unset", &format!("{:064b}", self.unset))
            .finish()
    }
}

impl CellOp {
    pub fn set(&mut self, val: Cell) {
        self.set |= val;
        self.unset &= !val;
    }
    pub fn unset(&mut self, val: Cell) {
        self.set &= !val;
        self.unset |= val;
    }
}

#[derive(Clone, Debug)]
pub struct BasicAnimation {
    cell: usize,
    visibility: CellOp,
    targeting: CellOp,
}

#[derive(Clone, Default, Debug)]
pub struct BasicContext {
    pub head: usize,
    pub animations: Vec<Vec<BasicAnimation>>,
}

#[derive(Clone, Default, Debug)]
pub struct BasicTimeline {
    pub main_context: BasicContext,
    pub contexts: Vec<BasicContext>,
}

pub struct Presentation {
    pub states_dyn: DoubleFilter,
    pub states_const: Vec<ShapeConstState>,
    pub timeline: BasicTimeline,
    pub target: (f32, f32),
    pub width: f32,
    pub height: f32,
    pub iters: usize,
}

impl Presentation {
    pub fn from(mut slide: Slide, target: (f32, f32)) -> Self {
        slide.shapes.sort_by(|a, b| a.1.z().cmp(&b.1.z()));
        let total_size = slide.shapes.iter().map(|e| e.1.size()).sum();
        let mut refs = vec![0; slide.shapes.len()];
        let mut shapes_dyn = DoubleFilter::new(total_size);
        let mut shapes_const = Vec::with_capacity(total_size);
        let mut referer_id = 0;
        let mut index = 0;
        for (id, shape) in slide.shapes.into_iter().rev() {
            match shape {
                Shape::Shape {
                    state: ShapeState { x, y, w, h, color },
                    ..
                } => {
                    refs[id] = referer_id;
                    shapes_dyn.set(index, true, target == (x, y));
                    shapes_const.push(ShapeConstState { color, x, y, w, h });
                    referer_id += 1;
                }
                Shape::Group { .. } => unimplemented!("groups"),
            };
            index += 1;
        }

        let len_full = slide
            .timeline
            .contexts
            .iter()
            .chain([&slide.timeline.main_context])
            .map(|c| c.animations.len())
            .sum::<usize>();
        let mut main_context = slide.timeline.main_context;
        let main_context = init_context(
            &mut main_context,
            &refs,
            &mut shapes_dyn,
            &mut shapes_const,
            target,
        );
        let mut contexts = vec![BasicContext::default(); total_size];
        for (id, mut context) in slide.timeline.contexts.into_iter().enumerate() {
            contexts[refs[id]] = init_context(
                &mut context,
                &refs,
                &mut shapes_dyn,
                &mut shapes_const,
                target,
            );
        }
        let len_basic = contexts
            .iter()
            .chain([&main_context])
            .map(|c| c.animations.iter().map(|s| s.len()).sum::<usize>())
            .sum::<usize>();
        println!("{len_full} {len_basic}");

        Presentation {
            timeline: BasicTimeline {
                main_context,
                contexts,
            },
            states_dyn: shapes_dyn,
            states_const: shapes_const,
            width: slide.width,
            height: slide.height,
            iters: 0,
            target,
        }
    }
}

impl fmt::Debug for Presentation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for state_const in &self.states_const {
            f.write_fmt(format_args!(
                "| {:04} {:04} {:04} {:04} ",
                state_const.x, state_const.y, state_const.w, state_const.h
            ))?;
        }
        for i in 0..self.states_const.len() {
            let (visible, targeted) = self.states_dyn.get(i);
            f.write_fmt(format_args!("| {visible:05} {targeted:05}         ",))?;
        }
        f.write_str("|\n")?;
        Ok(())
    }
}

impl Presentation {
    pub fn under(&mut self, x: f32, y: f32) -> Option<usize> {
        if self.target == (x, y) {
            return self.states_dyn.last();
        } else {
            for i in (0..self.states_const.len()).rev() {
                let (visible, targeted) = self.states_dyn.get(i);
                let state_const = &self.states_const[i];
                if visible {
                    let (sx, sy) = if targeted {
                        self.target
                    } else {
                        (state_const.x, state_const.y)
                    };
                    if x >= sx && y >= sy && x <= sx + state_const.w && y <= sy + state_const.h {
                        self.iters += self.states_const.len() - i;
                        return Some(i);
                    }
                }
            }
        }
        self.iters += self.states_const.len();
        return None;
    }

    pub fn click(&mut self, x: f32, y: f32) {
        let (target, context) = match self.under(x, y) {
            Some(referer) => {
                let context = &mut self.timeline.contexts[referer];
                if context.animations.is_empty() {
                    (None, &mut self.timeline.main_context)
                } else {
                    (Some(referer), context)
                }
            }
            None => (None, &mut self.timeline.main_context),
        };
        context.head = if context.head == context.animations.len() {
            if target.is_none() {
                exit(1);
            }
            1
        } else {
            context.head + 1
        };
        for animation in &context.animations[context.head - 1] {
            let cell = &mut self.states_dyn.cells[animation.cell];
            cell.0 |= animation.visibility.set;
            cell.0 &= !animation.visibility.unset;
            cell.1 |= animation.targeting.set;
            cell.1 &= !animation.targeting.unset;
        }
    }

    pub fn render(&self, scale: f32, background: Color) -> Canvas<Color> {
        let width = (self.width * scale) as usize;
        let height = (self.height * scale) as usize;
        let mut canvas = Canvas::new(width, height, background);
        for i in 0..self.states_const.len() {
            let (visible, targeted) = self.states_dyn.get(i);
            if visible {
                let ShapeConstState { color, x, y, w, h } = self.states_const[i];
                let (x, y) = if targeted { self.target } else { (x, y) };
                let x = (x * scale + 0.5) as isize;
                let y = (y * scale + 0.5) as isize;
                let w = (w * scale + 0.5) as isize;
                let h = (h * scale + 0.5) as isize;
                canvas.fill_rect(x, y, w, h, color);
            }
        }
        canvas
    }
}

pub fn init_context(
    context: &mut Context,
    refs: &[usize],
    shapes_dyn: &mut DoubleFilter,
    shapes_const: &mut [ShapeConstState],
    target_xy: (f32, f32),
) -> BasicContext {
    let mut animations = vec![];
    let mut cells = HashMap::new();
    for animation in context.animations.iter_mut() {
        if animation.click {
            let sequence = cells.into_values().collect::<Vec<_>>();
            cells = HashMap::new();
            animations.push(sequence);
        }
        let old_index = animation.target.index();
        let target = refs[old_index];
        let index = target >> CELL_SHIFT;
        let sub_index = 1 << (target & CELL_MASK);
        let effect = cells.entry(index).or_insert(BasicAnimation {
            cell: index,
            visibility: CellOp::default(),
            targeting: CellOp::default(),
        });
        let (ox, oy) = (shapes_const[target].x, shapes_const[target].y);
        match &animation.effect {
            Effect::Appear => {
                effect.visibility.set(sub_index);
            }
            Effect::Disappear => {
                effect.visibility.unset(sub_index);
            }
            Effect::SlideIn { .. } => {
                effect.visibility.set(sub_index);
                if target_xy == (ox, oy) {
                    effect.targeting.set(sub_index);
                } else {
                    effect.targeting.unset(sub_index);
                }
            }
            Effect::SlideOut { .. } => {
                effect.visibility.unset(sub_index);
                effect.targeting.unset(sub_index);
            }
            Effect::Path { x, y, relative, .. } => {
                let (x, y) = if *relative {
                    (ox + x, oy + y)
                } else {
                    (*x, *y)
                };
                if target_xy == (x, y) {
                    effect.targeting.set(sub_index);
                } else {
                    effect.targeting.unset(sub_index);
                }
            }
        }
    }
    let sequence = cells.into_values().collect::<Vec<_>>();
    animations.push(sequence);
    BasicContext {
        head: 0,
        animations,
    }
}
