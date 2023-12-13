use std::{fmt, process::exit};

use crate::{
    filters::Filter, render::Canvas, Color, Context, Referer, Shape, ShapeConstState,
    ShapeDynState, ShapeState, Slide, Timeline, Visibility,
};

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
    pub iters: usize,
    pub filter: Filter,
}

impl From<Slide> for Presentation {
    fn from(mut slide: Slide) -> Self {
        slide.shapes.sort_by(|a, b| a.1.z().cmp(&b.1.z()));
        let total_size = slide.shapes.iter().map(|e| e.1.size()).sum();
        let mut refs = vec![Referer::Shape(0); slide.shapes.len()];
        let mut shapes_dyn = Vec::with_capacity(total_size);
        let mut shapes_const = Vec::with_capacity(total_size);
        let mut shapes_groups = Vec::with_capacity(total_size);
        let mut referer_id = 0;
        for (id, shape) in slide.shapes.into_iter().rev() {
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

        let mut main_context = slide.timeline.main_context;
        main_context.init(&refs, &mut shapes_dyn, &mut shapes_const);
        let mut contexts = vec![Context::default(); total_size];
        for (id, mut context) in slide.timeline.contexts.into_iter().enumerate() {
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
            width: slide.width,
            height: slide.height,
            iters: 0,
            filter: Filter::new(total_size),
        }
    }
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
                self.iters += self.states_dyn.len() - index;
                return Some(self.referers[index]);
            }
        }
        self.iters += self.states_dyn.len();
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
                self.iters += start_index - index;
                self.cache_hit = CacheHit { x, y, index };
                return Some(self.referers[index]);
            }
        }
        self.iters += start_index;
        self.cache_hit = CacheHit { x, y, index: 0 };
        None
    }

    pub fn under_filter(&mut self, x: f32, y: f32) -> Option<Referer> {
        if self.cache_hit.x != x || self.cache_hit.y != y {
            return self.under(x, y);
        }
        self.filter.last().map(|index| self.referers[index])
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
                if obstructible
                    && i > cache_index
                    && self.states_dyn[i].contains(self.cache_hit.x, self.cache_hit.y)
                {
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

    pub fn click_filter(&mut self, x: f32, y: f32) {
        let (target, context) = match self.under_filter(x, y) {
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
                let state = &self.states_dyn[i];
                if obstructible && state.contains(self.cache_hit.x, self.cache_hit.y) {
                    self.filter.set(i);
                } else {
                    self.filter.unset(i);
                }
                if perceptible && i < cache_min {
                    cache_min = i;
                }
                if perceptible && i > cache_max {
                    cache_max = i;
                }
            }
        }
        self.cache_data.start = cache_min;
        self.cache_data.end = cache_max;
        self.cache_data.update = true;
    }

    pub fn update_filter(&mut self, x: f32, y: f32) {
        self.cache_hit.x = x;
        self.cache_hit.y = y;
        for (i, state) in self.states_dyn.iter().enumerate() {
            if state.is_visible() && state.contains(x, y) {
                self.filter.set(i);
            } else {
                self.filter.unset(i);
            }
        }
    }

    pub fn render(&self, scale: f32, background: Color) -> Canvas<Color> {
        let width = (self.width * scale) as usize;
        let height = (self.height * scale) as usize;
        let mut canvas = Canvas::new(width, height, background);
        for i in 0..self.states_dyn.len() {
            let ShapeDynState {
                x,
                y,
                w,
                h,
                visibiliy,
            } = self.states_dyn[i];
            if visibiliy.is_visible() {
                let ShapeConstState { color, .. } = self.states_const[i];
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
