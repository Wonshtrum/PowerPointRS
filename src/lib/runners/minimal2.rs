use std::{fmt, process::exit};

use crate::{
    filters::Filter, render::Canvas, Color, Context, Effect, Preset, Referer, Shape, ShapeState,
    Slide, Timeline,
};

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
pub struct ShapeConstState {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: Color,
}

#[derive(Clone)]
#[repr(C)]
pub struct ShapeDynState {
    pub visible: bool,
    pub targeted: bool,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct CacheData {
    update: bool,
    start: usize,
    end: usize,
}

#[derive(Clone, Debug)]
enum Change {
    True,
    False,
    Same,
}

impl From<bool> for Change {
    fn from(change: bool) -> Self {
        if change {
            Change::True
        } else {
            Change::False
        }
    }
}

#[derive(Clone, Debug)]
pub struct BasicAnimation {
    target: usize,
    visible: Change,
    targeted: Change,
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
    pub states_dyn: Vec<ShapeDynState>,
    pub states_const: Vec<ShapeConstState>,
    pub timeline: BasicTimeline,
    pub cache_data: CacheData,
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
        let mut shapes_dyn = Vec::with_capacity(total_size);
        let mut shapes_const = Vec::with_capacity(total_size);
        let mut referer_id = 0;
        for (id, shape) in slide.shapes.into_iter().rev() {
            match shape {
                Shape::Shape {
                    state: ShapeState { x, y, w, h, color },
                    ..
                } => {
                    refs[id] = referer_id;
                    shapes_dyn.push(ShapeDynState {
                        visible: true,
                        targeted: target == (x, y),
                    });
                    shapes_const.push(ShapeConstState { color, x, y, w, h });
                    referer_id += 1;
                }
                Shape::Group { .. } => unimplemented!("groups"),
            };
        }

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

        Presentation {
            timeline: BasicTimeline {
                main_context,
                contexts,
            },
            cache_data: CacheData {
                update: false,
                start: 0,
                end: shapes_dyn.len(),
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
        f.write_fmt(format_args!("cache_data: {:?}\n", self.cache_data,))?;
        for state_const in &self.states_const {
            f.write_fmt(format_args!(
                "| {:04} {:04} {:04} {:04} ",
                state_const.x, state_const.y, state_const.w, state_const.h
            ))?;
        }
        for state_dyn in &self.states_dyn {
            f.write_fmt(format_args!(
                "| {:05} {:05}         ",
                state_dyn.visible, state_dyn.targeted
            ))?;
        }
        f.write_str("|\n")?;
        Ok(())
    }
}

impl Presentation {
    pub fn under(&mut self, x: f32, y: f32) -> Option<usize> {
        if self.target == (x, y) {
            for (index, state) in self.states_dyn.iter().enumerate().rev() {
                if state.visible && state.targeted {
                    self.iters += self.states_dyn.len() - index;
                    return Some(index);
                }
            }
        } else {
            for i in (0..self.states_const.len()).rev() {
                let state_dyn = &self.states_dyn[i];
                let state_const = &self.states_const[i];
                if state_dyn.visible {
                    let (sx, sy) = if state_dyn.targeted {
                        self.target
                    } else {
                        (state_const.x, state_const.y)
                    };
                    if x >= sx && y >= sy && x <= sx + state_const.w && y <= sy + state_const.h {
                        self.iters += self.states_dyn.len() - i;
                        return Some(i);
                    }
                }
            }
        }
        self.iters += self.states_dyn.len();
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
        // let (mut cache_min, mut cache_max) = if self.cache_data.update {
        //     (self.cache_data.start, self.cache_data.end)
        // } else {
        //     (self.states_dyn.len(), 0)
        // };
        for animation in context.animations[context.head - 1].iter_mut() {
            let target = animation.target;
            let perceptible = apply_effect(
                animation,
                &mut self.states_dyn[target],
                &self.states_const[target],
            );
            // if perceptible && target < cache_min {
            //     cache_min = target;
            // }
            // if perceptible && target > cache_max {
            //     cache_max = target;
            // }
        }
        // self.cache_data.start = cache_min;
        // self.cache_data.end = cache_max;
        // self.cache_data.update = true;
    }

    pub fn render(&self, scale: f32, background: Color) -> Canvas<Color> {
        let width = (self.width * scale) as usize;
        let height = (self.height * scale) as usize;
        let mut canvas = Canvas::new(width, height, background);
        for i in 0..self.states_dyn.len() {
            let ShapeDynState { visible, targeted } = self.states_dyn[i];
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

fn apply_effect(
    effect: &mut BasicAnimation,
    state_dyn: &mut ShapeDynState,
    state_const: &ShapeConstState,
) -> bool {
    let old_visibility = state_dyn.visible;
    let old_targeting = state_dyn.targeted;
    match effect.visible {
        Change::True => state_dyn.visible = true,
        Change::False => state_dyn.visible = false,
        Change::Same => {}
    }
    match effect.targeted {
        Change::True => state_dyn.targeted = true,
        Change::False => state_dyn.targeted = false,
        Change::Same => {}
    }
    old_visibility != state_dyn.visible
        || (state_dyn.visible && old_targeting != state_dyn.targeted)
}

fn init_context(
    context: &mut Context,
    refs: &[usize],
    shapes_dyn: &mut [ShapeDynState],
    shapes_const: &mut [ShapeConstState],
    target_xy: (f32, f32),
) -> BasicContext {
    let mut animations = vec![];
    for animation in &mut context.animations {
        if animation.click || animations.is_empty() {
            animations.push(vec![]);
        }
        let old_index = animation.target.index();
        let target = refs[old_index];
        let (ox, oy) = (shapes_const[target].x, shapes_const[target].y);
        animations
            .last_mut()
            .unwrap()
            .push(match &animation.effect {
                Effect::Appear => BasicAnimation {
                    target,
                    visible: Change::True,
                    targeted: Change::Same,
                },
                Effect::Disappear => BasicAnimation {
                    target,
                    visible: Change::False,
                    targeted: Change::Same,
                },
                Effect::SlideIn { .. } => BasicAnimation {
                    target,
                    visible: Change::True,
                    targeted: Change::from(target_xy == (ox, oy)),
                },
                Effect::SlideOut { .. } => BasicAnimation {
                    target,
                    visible: Change::False,
                    targeted: Change::False,
                },
                Effect::Path { x, y, relative, .. } => {
                    let (x, y) = if *relative {
                        (ox + x, oy + y)
                    } else {
                        (*x, *y)
                    };
                    BasicAnimation {
                        target,
                        visible: Change::Same,
                        targeted: Change::from(target_xy == (x, y)),
                    }
                }
            })
        // let (start, end) = target.bounds();
        // for state_dyn in &mut shapes_dyn[start..end] {
        //     match (animation.effect.preset(), state_dyn.visibility) {
        //         (Preset::Entr(_, _), Visibility::Unknown) => {
        //             state_dyn.visibility = Visibility::Hidden
        //         }
        //         (_, Visibility::Unknown) => state_dyn.visibility = Visibility::Visible,
        //         _ => {}
        //     }
        // }
    }
    BasicContext {
        head: 0,
        animations,
    }
}
