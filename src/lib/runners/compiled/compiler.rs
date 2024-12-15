use std::collections::{HashMap, VecDeque};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::{fmt, mem};

use crate::bitvec::{BinaryFmt, BitVec, Bits};
use crate::runners::compiled::{render_state, RenderingContext};
use crate::{pause, Color};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tri {
    /// True
    T,
    /// False
    F,
    /// Unknown or Unchanged
    U,
}

#[derive(Clone)]
pub struct Action<T: Bits> {
    pub chunk: usize,
    pub set_t: T,
    pub unset_t: T,
    pub set_v: T,
    pub unset_v: T,
}
impl<T: Bits> fmt::Debug for Action<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Action")
            .field("chunk", &self.chunk)
            .field("set_t", &BinaryFmt(self.set_t))
            .field("unset_t", &BinaryFmt(self.unset_t))
            .field("set_v", &BinaryFmt(self.set_v))
            .field("unset_v", &BinaryFmt(self.unset_v))
            .finish()
    }
}
impl<T: Bits> Action<T> {
    pub fn new(chunk: usize) -> Self {
        Self {
            chunk,
            set_t: T::ZERO,
            unset_t: T::ZERO,
            set_v: T::ZERO,
            unset_v: T::ZERO,
        }
    }
}

#[derive(Debug)]
pub struct Context<T: Bits> {
    pub n: usize,
    pub cap: usize,
    pub actions: Vec<Vec<Action<T>>>,
    pub root: State<T>,
    pub u: BitVec<T>,
}
impl<T: Bits> Context<T> {
    pub fn new(n: usize) -> Self {
        let cap = match n % T::SIZE {
            0 => n / T::SIZE,
            _ => 1 + (n / T::SIZE),
        };
        Self {
            n,
            cap,
            actions: vec![Vec::new(); n],
            root: State::new(cap),
            u: BitVec::new(cap),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct State<T: Bits> {
    /// targeted
    pub t: BitVec<T>,
    /// visible
    pub v: BitVec<T>,
    /// unknown visibility (override visible)
    pub u: BitVec<T>,
}
impl<T: Bits> State<T> {
    pub fn new(cap: usize) -> Self {
        Self {
            t: BitVec::new(cap),
            v: BitVec::new(cap),
            u: BitVec::new(cap),
        }
    }
    pub fn firsts(&self, firsts: &mut Vec<ShapeId>) {
        firsts.clear();
        for (i, u) in self.u.iter().copied().enumerate() {
            let mut tv = self.t.get_chunk(i) & self.v.get_chunk(i);
            let mut offset = 0;
            while tv != T::ZERO {
                let o = tv.trailing_zeros();
                offset += o;
                firsts.push(ShapeId::new(i * T::SIZE + offset));
                if !u.get(offset) {
                    return;
                }
                if o == 7 {
                    break;
                }
                offset += 1;
                tv = tv >> (o + 1);
            }
        }
    }
    pub fn apply(&mut self, ctx: &Context<T>, src: ShapeId, prevs: &[ShapeId]) {
        for prev in prevs {
            self.v.unset(prev.index());
            self.u.unset(prev.index());
        }
        for action in &ctx.actions[src.index()] {
            let i = action.chunk;
            *self.t.get_chunk_mut(i) |= action.set_t;
            *self.t.get_chunk_mut(i) &= !action.unset_t;
            *self.v.get_chunk_mut(i) |= action.set_v;
            *self.v.get_chunk_mut(i) &= !action.unset_v;
            *self.u.get_chunk_mut(i) &= !action.set_v | !action.unset_v;
        }
    }
    pub fn can_merge(&self, other: &Self) -> bool {
        for (i, u) in self.u.iter().copied().enumerate() {
            let t = self.t.get_chunk(i);
            let v = self.v.get_chunk(i);
            let o_t = other.t.get_chunk(i);
            let o_v = other.v.get_chunk(i);
            let o_u = other.u.get_chunk(i);
            let same_t = t & !u == o_t & !u;
            let same_v = v & !u == o_v & !u;
            let included_u = (!u & o_u) == T::ZERO;
            if !same_t || !same_v || !included_u {
                return false;
            }
        }
        true
    }
}

pub fn partial_hash<T: Bits>(t: &[T], v: &[T]) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    v.hash(&mut hasher);
    hasher.finish()
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(u32);
impl NodeId {
    pub fn new<T: TryInto<u32>>(x: T) -> Self {
        Self(x.try_into().ok().unwrap())
    }
    pub fn index(self) -> usize {
        self.0 as usize
    }
}
impl fmt::Debug for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShapeId(u32);
impl ShapeId {
    pub fn new<T: TryInto<u32>>(x: T) -> Self {
        Self(x.try_into().ok().unwrap())
    }
    pub fn index(self) -> usize {
        self.0 as usize
    }
}
impl fmt::Debug for ShapeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy)]
pub struct Edge(NodeId, ShapeId);
impl fmt::Debug for Edge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:?}, {:?})", self.0, self.1)
    }
}
#[derive(Debug, Clone)]
pub enum Edges {
    Zero,
    One([Edge; 1]),
    Two([Edge; 2]),
    Many(Vec<Edge>),
}
impl Edges {
    pub fn push(&mut self, node_id: NodeId, shape_id: ShapeId) {
        let edge = Edge(node_id, shape_id);
        match self {
            Self::Zero => *self = Self::One([edge]),
            Self::One([e0]) => *self = Self::Two([*e0, edge]),
            Self::Two([e0, e1]) => *self = Self::Many(vec![*e0, *e1, edge]),
            Self::Many(v) => v.push(edge),
        }
    }
    pub fn as_slice(&self) -> &[Edge] {
        match self {
            Self::Zero => &[],
            Self::One(edges) => edges,
            Self::Two(edges) => edges,
            Self::Many(edges) => edges,
        }
    }
}

#[derive(Debug)]
pub struct Node {
    pub from: Edges,
}
impl Node {
    pub fn new(parent_id: NodeId, shape_id: ShapeId) -> Self {
        Self {
            from: Edges::One([Edge(parent_id, shape_id)]),
        }
    }
}

#[derive(Debug)]
pub struct Arena<T: Bits> {
    set_t: BitVec<T>,
    unset_t: BitVec<T>,
    set_v: BitVec<T>,
    unset_v: BitVec<T>,
}
impl<T: Bits> Arena<T> {
    pub fn new(cap: usize) -> Self {
        Self {
            set_t: BitVec::new(cap),
            unset_t: BitVec::new(cap),
            set_v: BitVec::new(cap),
            unset_v: BitVec::new(cap),
        }
    }
}

#[derive(Debug)]
pub struct Graph<T: Bits> {
    pub nodes: Vec<Node>,
    pub cache: HashMap<NodeId, State<T>>,
    pub hashes: HashMap<u64, Vec<NodeId>>,
}
impl<T: Bits> Graph<T> {
    pub fn new(ctx: &Context<T>) -> Self {
        let mut graph = Self {
            nodes: vec![Node { from: Edges::Zero }],
            cache: HashMap::new(),
            hashes: HashMap::new(),
        };
        graph.update_filters(ctx, &ctx.root, NodeId(0));
        graph
    }
    pub fn merge(
        &mut self,
        ctx: &Context<T>,
        arena: &mut Arena<T>,
        state: &mut State<T>,
    ) -> Option<NodeId> {
        println!("trying to merge: {state:#?}");
        let h = partial_hash(&state.t, &state.v);
        if let Some(candidates) = self.hashes.get(&h) {
            println!("candidates: {candidates:?}");
            for candidate_id in candidates {
                if let Some(candidate) = self.cache.get(candidate_id) {
                    println!("cache hit");
                    if candidate.can_merge(state) {
                        println!("CAN MERGE INTO CANDIDATE");
                        return Some(*candidate_id);
                    }
                    println!("CANNOT MERGE INTO CANDIDATE");
                } else {
                    println!("cache miss");
                    let computed = self.compute_state(ctx, arena, *candidate_id);
                    println!("COMPUTED: {computed:#?}");
                    if computed.can_merge(state) {
                        println!("CAN MERGE INTO COMPUTED");
                        return Some(*candidate_id);
                    }
                    println!("CANNOT MERGE INTO COMPUTED")
                    // do something with computed
                }
            }
        } else {
            println!("UNKNOWN PARTIAL HASH");
        }
        None
    }
    pub fn compute_state(
        &self,
        ctx: &Context<T>,
        arena: &mut Arena<T>,
        mut node_id: NodeId,
    ) -> State<T> {
        let Arena {
            set_t,
            unset_t,
            set_v,
            unset_v,
        } = arena;
        set_t.clear();
        unset_t.clear();
        set_v.copy(&ctx.root.u);
        unset_v.clear();
        let mut state = 'next: loop {
            println!("following {node_id:?}");
            if node_id == NodeId(0) {
                break ctx.root.clone();
            }
            if let Some(state) = self.cache.get(&node_id) {
                break state.clone();
            }
            for Edge(parent_id, shape_id) in self.nodes[node_id.index()].from.as_slice() {
                if *parent_id < node_id {
                    node_id = *parent_id;
                    for action in &ctx.actions[shape_id.index()] {
                        let i = action.chunk;
                        *set_t.get_chunk_mut(i) |= action.set_t & !unset_t.get_chunk(i);
                        *unset_t.get_chunk_mut(i) |= action.unset_t & !set_t.get_chunk(i);
                        *set_v.get_chunk_mut(i) |= action.set_v & !unset_v.get_chunk(i);
                        *unset_v.get_chunk_mut(i) |= action.unset_v & !set_v.get_chunk(i);
                    }
                    continue 'next;
                }
            }
            panic!("node has no ancestor");
        };
        for i in 0..ctx.cap {
            *state.t.get_chunk_mut(i) |= set_t.get_chunk(i);
            *state.t.get_chunk_mut(i) &= !unset_t.get_chunk(i);
            *state.v.get_chunk_mut(i) |= set_v.get_chunk(i);
            *state.v.get_chunk_mut(i) &= !unset_v.get_chunk(i);
            *state.u.get_chunk_mut(i) &= !set_v.get_chunk(i) | !unset_v.get_chunk(i);
        }
        state
    }
    fn push(
        &mut self,
        ctx: &Context<T>,
        state: &State<T>,
        parent_id: NodeId,
        shape_id: ShapeId,
    ) -> NodeId {
        let node_id = NodeId::new(self.nodes.len());
        self.nodes.push(Node::new(parent_id, shape_id));
        self.update_filters(ctx, state, node_id);
        node_id
    }
    fn update_filters(&mut self, ctx: &Context<T>, state: &State<T>, node_id: NodeId) {
        let mut v = state.v.clone();
        v.binop(&ctx.u, |a, b| *a |= b);
        let h = partial_hash(&state.t, &v);
        self.hashes.entry(h).or_default().push(node_id);
        // update cache?
    }
}

pub fn compile<T: Bits>(ctx: &Context<T>, render_ctx: RenderingContext) -> Graph<T> {
    let mut graph = Graph::new(ctx);
    let mut arena = Arena::new(ctx.cap);
    let mut states = VecDeque::from([(NodeId(0), ctx.root.clone())]);
    let mut firsts = Vec::new();

    while let Some((parent_id, mut parent)) = states.pop_front() {
        pause();
        println!(
            "{}",
            render_state(&render_ctx, &parent, 1.0, Color::new(230, 230, 255))
        );
        pause();
        parent.firsts(&mut firsts);
        for (i, node) in graph.nodes.iter().enumerate() {
            println!("{i} <- {:?}", node.from.as_slice());
        }
        println!("{parent_id:?}: {parent:#?}");
        println!("firsts: {firsts:?}");
        for (i, first) in firsts.iter().copied().enumerate() {
            let mut next = if i == firsts.len() - 1 {
                mem::take(&mut parent)
            } else {
                println!("CLONING NEXT");
                let next = parent.clone();
                next
            };
            next.apply(ctx, first, &firsts[..i]);
            if let Some(next_id) = graph.merge(ctx, &mut arena, &mut next) {
                graph.nodes[next_id.index()].from.push(parent_id, first);
            } else {
                let next_id = graph.push(ctx, &next, parent_id, first);
                states.push_back((next_id, next));
            }
        }
    }
    println!("{graph:#?}");
    graph
}
