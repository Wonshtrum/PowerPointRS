use std::collections::HashMap;

pub mod compiler;

use crate::{
    bitvec::{BitVec, Bits},
    render::Canvas,
    runners::compiled::compiler as cc,
    Color, Effect, Preset, Shape, ShapeState, Slide,
};
use cc::Tri;

#[derive(Clone)]
#[repr(C)]
pub struct ShapeConstState {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: Color,
}

pub struct Presentation<T: Bits> {
    pub context: cc::Context<T>,
    pub graph: cc::Graph<T>,
    pub state: cc::State<T>,
    pub states_const: Vec<ShapeConstState>,
    pub width: f32,
    pub height: f32,
    pub iters: usize,
    pub target: (f32, f32),
}

impl<T: Bits> Presentation<T> {
    pub fn from(mut slide: Slide, target: (f32, f32)) -> Self {
        slide.shapes.sort_by_key(|s| s.1.z());
        let total_size = slide.shapes.iter().map(|e| e.1.size()).sum();
        let mut refs = vec![0; slide.shapes.len()];
        let mut states_const = Vec::with_capacity(total_size);

        let mut ctx = cc::Context::new(total_size);
        let mut t = BitVec::new(ctx.cap);
        let mut u = BitVec::new(ctx.cap);
        let mut tmp_v = Vec::with_capacity(total_size);

        for (referer_id, (id, shape)) in slide.shapes.into_iter().enumerate() {
            match shape {
                Shape::Shape {
                    state: ShapeState { x, y, w, h, color },
                    name,
                    ..
                } => {
                    refs[id] = referer_id;
                    if target == (x, y) {
                        t.set(referer_id);
                    }
                    tmp_v.push(Tri::U);
                    if name.contains("UNKNOWN") || name.contains("CONTROL") {
                        u.set(referer_id);
                    }
                    states_const.push(ShapeConstState { color, x, y, w, h });
                }
                Shape::Group { .. } => unimplemented!("groups"),
            };
        }

        for animation in slide.timeline.main_context.animations {
            if animation.click {
                unimplemented!("animation click");
            }
            let index = refs[animation.target.index()];
            let (ox, oy) = (states_const[index].x, states_const[index].y);
            match animation.effect {
                Effect::Appear => tmp_v[index] = Tri::T,
                Effect::Disappear => tmp_v[index] = Tri::F,
                Effect::SlideIn { .. } => {
                    tmp_v[index] = Tri::T;
                    t.set(index);
                }
                Effect::SlideOut { complete, .. } => {
                    if !complete {
                        unimplemented!("incomplete SlideOut");
                    }
                    tmp_v[index] = Tri::F;
                    if target == (ox, oy) {
                        t.set(index);
                    } else {
                        t.unset(index);
                    }
                }
                Effect::Path { x, y, relative, .. } => {
                    let (x, y) = if relative { (ox + x, oy + y) } else { (x, y) };
                    if target == (x, y) {
                        t.set(index);
                    } else {
                        t.unset(index);
                    }
                }
            }
        }
        for (id, context) in slide.timeline.contexts.into_iter().enumerate() {
            let mut actions = HashMap::<usize, cc::Action<T>>::new();
            for animation in context.animations {
                if animation.click {
                    //unimplemented!("animation click");
                }
                let index = refs[animation.target.index()];
                match (animation.effect.preset(), tmp_v[index]) {
                    (Preset::Entr(..), Tri::U) => tmp_v[index] = Tri::F,
                    (_, Tri::U) => tmp_v[index] = Tri::T,
                    _ => {}
                }
                let (ox, oy) = (states_const[index].x, states_const[index].y);
                let chunk = index / T::SIZE;
                let index = index % T::SIZE;
                let action = actions.entry(chunk).or_insert(cc::Action::new(chunk));
                match animation.effect {
                    Effect::Appear => action.set_v.set(index),
                    Effect::Disappear => action.unset_v.set(index),
                    Effect::SlideIn { .. } => {
                        action.set_v.set(index);
                        if target == (ox, oy) {
                            action.set_t.set(index);
                        } else {
                            action.unset_t.set(index);
                        }
                    }
                    Effect::SlideOut { complete, .. } => {
                        if !complete {
                            unimplemented!("incomplete SlideOut");
                        }
                        action.unset_v.set(index);
                        if target == (ox, oy) {
                            action.set_t.set(index);
                        } else {
                            action.unset_t.set(index);
                        }
                    }
                    Effect::Path { x, y, relative, .. } => {
                        let (x, y) = if relative { (ox + x, oy + y) } else { (x, y) };
                        if target == (x, y) {
                            action.set_t.set(index);
                        } else {
                            action.unset_t.set(index);
                        }
                    }
                }
            }
            ctx.actions[refs[id]] = actions.into_values().collect::<Vec<_>>();
            ctx.actions[refs[id]].sort_by_key(|a| a.chunk);
        }

        let mut v = BitVec::new(ctx.cap);
        for (i, v_or_u) in tmp_v.into_iter().enumerate() {
            match v_or_u {
                Tri::T | Tri::U => v.set(i),
                Tri::F => {}
            }
        }

        ctx.root.t.copy(&t);
        ctx.root.u.copy(&u);
        ctx.root.v.copy(&v);
        ctx.root.v.binop(&u, |a, b| *a |= b);
        let state = cc::State { t, v, u };

        println!("{:#?}", ctx.root);
        println!("{state:#?}");

        let render_ctx = RenderingContext {
            states_const: &states_const,
            target,
            width: slide.width,
            height: slide.height,
        };
        let graph = cc::compile(&ctx, render_ctx);
        Presentation {
            context: ctx,
            graph,
            state,
            target,
            states_const,
            width: slide.width,
            height: slide.height,
            iters: 0,
        }
    }
}

impl<T: Bits> Presentation<T> {
    #[allow(unused_variables)]
    pub fn under(&mut self, x: f32, y: f32) -> Option<usize> {
        todo!()
    }

    #[allow(unused_variables)]
    pub fn click(&mut self, x: f32, y: f32) {
        todo!()
    }

    pub fn render(&self, scale: f32, background: Color) -> Canvas<Color> {
        render_state(
            &RenderingContext {
                states_const: &self.states_const,
                target: self.target,
                width: self.width,
                height: self.height,
            },
            &self.state,
            scale,
            background,
        )
    }
}

pub struct RenderingContext<'a> {
    states_const: &'a [ShapeConstState],
    target: (f32, f32),
    width: f32,
    height: f32,
}

pub fn render_state<T: Bits>(
    ctx: &RenderingContext,
    state: &cc::State<T>,
    scale: f32,
    background: Color,
) -> Canvas<Color> {
    let width = (ctx.width * scale) as usize;
    let height = (ctx.height * scale) as usize;
    let mut canvas = Canvas::new(width, height, background);
    for (i, state_const) in ctx.states_const.iter().enumerate().rev() {
        let targeted = state.t.get(i);
        let visible = state.v.get(i);
        let unknown = state.u.get(i);
        if visible {
            let ShapeConstState { color, x, y, w, h } = *state_const;
            let (x, y) = if targeted { ctx.target } else { (x, y) };
            let x = (x * scale + 0.5) as isize;
            let y = (y * scale + 0.5) as isize;
            let w = (w * scale + 0.5) as isize;
            let h = (h * scale + 0.5) as isize;
            if unknown && (targeted || (w > 1 && h > 1)) {
                canvas.fill_rect(x - 1, y - 1, w + 2, h + 2, Color::new(255, 255, 0));
            }
            canvas.fill_rect(x, y, w, h, color);
        }
    }
    canvas
}
