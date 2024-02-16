use std::collections::HashMap;

mod compiler;

use crate::{
    filters::DoubleFilter, render::Canvas, runners::compiled::compiler as cc, Color, Context,
    Effect, Preset, Shape, ShapeState, Slide,
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

#[derive(Clone, Debug)]
pub struct CacheHit {
    pub x: f32,
    pub y: f32,
    pub index: usize,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct CacheData {
    pub update: bool,
    pub start: usize,
    pub end: usize,
}

pub struct Presentation {
    pub states_dyn: DoubleFilter,
    pub states_const: Vec<ShapeConstState>,
    pub target: (f32, f32),
    pub width: f32,
    pub height: f32,
    pub iters: usize,
    pub compiler: cc::Compiler,
}

impl Presentation {
    pub fn from(mut slide: Slide, target: (f32, f32)) -> Self {
        slide.shapes.sort_by(|a, b| a.1.z().cmp(&b.1.z()));
        let total_size = slide.shapes.iter().map(|e| e.1.size()).sum();
        let mut refs = vec![0; slide.shapes.len()];
        let mut states_dyn = DoubleFilter::new(total_size);
        let mut states_const = Vec::with_capacity(total_size);
        let mut elements = Vec::with_capacity(total_size);
        for (referer_id, (id, shape)) in slide.shapes.into_iter().enumerate() {
            match shape {
                Shape::Shape {
                    state: ShapeState { x, y, w, h, color },
                    name,
                    ..
                } => {
                    refs[id] = referer_id;
                    states_const.push(ShapeConstState { color, x, y, w, h });
                    elements.push(cc::Element {
                        name,
                        v: cc::State::Unknown,
                        t: (target == (x, y)).into(),
                        actions: Vec::new(),
                    });
                }
                Shape::Group { .. } => unimplemented!("groups"),
            };
        }

        let len_full = slide
            .timeline
            .contexts
            .iter()
            .chain([&slide.timeline.main_context])
            .map(|c| c.animations.len())
            .sum::<usize>();

        let mut main_context = slide.timeline.main_context;
        let main_actions = init_context(
            &mut main_context,
            &refs,
            &mut elements,
            &mut states_const,
            target,
        );
        for element in &mut elements {
            element.v = element.v.unwrap_or(true).into();
        }
        for action in &main_actions {
            match action.change {
                cc::Change::V(val) => elements[action.index].v = val.into(),
                cc::Change::T(val) => elements[action.index].t = val.into(),
            }
        }
        elements.push(cc::Element::BG);
        println!("{elements:#?}");
        let mut cc_states_const = elements
            .iter()
            .map(|e| cc::ElementConstState {
                name: e.name,
                actions: Vec::new(),
            })
            .collect::<Vec<_>>();
        for (id, mut context) in slide.timeline.contexts.into_iter().enumerate() {
            let actions = init_context(
                &mut context,
                &refs,
                &mut elements,
                &mut states_const,
                target,
            );
            cc_states_const[refs[id]].actions = actions;
        }

        let len_compiled = cc_states_const
            .iter()
            .map(|s| s.actions.len())
            .sum::<usize>()
            + main_actions.len();
        println!("{len_full} {len_compiled}");

        let compiler = cc::Compiler {
            context: cc::Context {
                states_const: cc_states_const,
            },
            node: cc::Node::new("n0", &elements),
        };

        let mut next = compiler.node.next(&compiler.context);
        println!("{next:#?}");

        for (i, element) in elements.iter_mut().enumerate() {
            let v = element.v.unwrap_or(true);
            let t = element.t.unwrap_or(true);
            states_dyn.set(i, v, t);
        }
        apply_node(next.pop().unwrap(), &mut states_dyn);

        Presentation {
            states_dyn,
            states_const,
            width: slide.width,
            height: slide.height,
            iters: 0,
            target,
            compiler,
        }
    }
}

fn apply_node(node: cc::Node, states_dyn: &mut DoubleFilter) {
    println!("applying: {}", node.name);
    for (i, element) in node.states_dyn.iter().enumerate() {
        let v = element.v.unwrap_or(true);
        let t = element.t.unwrap_or(true);
        states_dyn.set(i, v, t);
    }
}

impl Presentation {
    pub fn click(&mut self, x: f32, y: f32) {}

    pub fn render(&self, scale: f32, background: Color) -> Canvas<Color> {
        let width = (self.width * scale) as usize;
        let height = (self.height * scale) as usize;
        let mut canvas = Canvas::new(width, height, background);
        for i in (0..self.states_const.len()).rev() {
            let (visible, targeted) = self.states_dyn.get(i);
            if self.compiler.context.states_const[i].name == "START" {
                println!("> {i} {visible} {targeted}");
            }
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
    elements: &mut [cc::Element],
    states_const: &mut [ShapeConstState],
    target_xy: (f32, f32),
) -> Vec<cc::Action> {
    let mut states_v = HashMap::new();
    let mut states_t = HashMap::new();
    for animation in &mut context.animations {
        if animation.click {
            // unimplemented!("on click animation");
        }
        let old_index = animation.target.index();
        let target = refs[old_index];

        let (ox, oy) = (states_const[target].x, states_const[target].y);
        match &animation.effect {
            Effect::Appear => {
                states_v.insert(target, true);
            }
            Effect::Disappear => {
                states_v.insert(target, false);
            }
            Effect::SlideIn { .. } => {
                states_v.insert(target, true);
                if target_xy == (ox, oy) {
                    states_t.insert(target, true);
                } else {
                    states_t.insert(target, false);
                }
            }
            Effect::SlideOut { complete, .. } => {
                if !complete {
                    unimplemented!("incomplete SlideOut");
                }
                states_v.insert(target, false);
                states_t.insert(target, false);
            }
            Effect::Path { x, y, relative, .. } => {
                let (x, y) = if *relative {
                    (ox + x, oy + y)
                } else {
                    (*x, *y)
                };
                if target_xy == (x, y) {
                    states_t.insert(target, true);
                } else {
                    states_t.insert(target, false);
                }
            }
        }
        match (animation.effect.preset(), elements[target].v) {
            (Preset::Entr(_, _), cc::State::Unknown) => {
                elements[target].v = cc::State::False;
                println!("Hide {target}: {:#?}", elements[target]);
            }
            (_, cc::State::Unknown) => elements[target].v = cc::State::True,
            _ => {}
        }
    }

    let iter_t = states_t
        .into_iter()
        .map(|(target, value)| cc::Action::change_t(target, value));
    let iter_v = states_v
        .into_iter()
        .map(|(target, value)| cc::Action::change_v(target, value));
    iter_t.chain(iter_v).collect()
}
