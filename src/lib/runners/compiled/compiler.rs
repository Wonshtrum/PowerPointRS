#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    True,
    False,
    Unknown,
}

impl State {
    pub fn can_merge(self, other: Self) -> bool {
        match (self, other) {
            (Self::True, Self::False) | (Self::False, Self::True) => false,
            _ => true,
        }
    }

    pub fn unwrap_or(self, default: bool) -> bool {
        match self {
            State::True => true,
            State::False => false,
            State::Unknown => default,
        }
    }
}

impl From<bool> for State {
    fn from(value: bool) -> Self {
        if value {
            State::True
        } else {
            State::False
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Change {
    V(bool),
    T(bool),
}

#[derive(Clone, Debug)]
pub struct Action {
    pub index: usize,
    pub change: Change,
}

impl Action {
    pub fn change_v(index: usize, value: bool) -> Self {
        Self {
            index,
            change: Change::V(value),
        }
    }
    pub fn change_t(index: usize, value: bool) -> Self {
        Self {
            index,
            change: Change::T(value),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ElementDynState {
    pub v: State,
    pub t: State,
    pub weak: bool,
}

#[derive(Clone, Debug)]
pub struct ElementConstState {
    pub name: &'static str,
    pub actions: Vec<Action>,
}

impl ElementDynState {
    pub fn can_merge(&self, other: &Self) -> bool {
        if true && self.weak {
            return other.weak
                || match (other.v, other.t) {
                    (State::False, State::False) // may be too strong
                    | (State::True, State::False)
                    | (State::False, State::True) => true,
                    _ => false,
                };
        }
        self.t.can_merge(other.t) && self.v.can_merge(other.v)
    }
}

#[derive(Clone, Debug)]
pub struct Element {
    pub name: &'static str,
    pub v: State,
    pub t: State,
    pub actions: Vec<Action>,
}

impl Element {
    pub const BG: Self = Self {
        name: "bg",
        v: State::True,
        t: State::True,
        actions: Vec::new(),
    };

    pub fn state(&self) -> ElementDynState {
        ElementDynState {
            v: self.v,
            t: self.t,
            weak: false,
        }
    }
}

#[derive(Debug)]
pub struct Context {
    pub states_const: Vec<ElementConstState>,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub name: String,
    pub states_dyn: Vec<ElementDynState>,
    pub finished: bool,
}

#[derive(Debug)]
pub struct Compiler {
    pub context: Context,
    pub node: Node,
}

impl Node {
    pub fn new<S: Into<String>>(name: S, elements: &[Element]) -> Self {
        Self {
            name: name.into(),
            states_dyn: elements.iter().map(Element::state).collect(),
            finished: false,
        }
    }
    pub fn can_merge(&self, other: &Self) -> bool {
        self.states_dyn
            .iter()
            .zip(&other.states_dyn)
            .all(|(e, o)| e.can_merge(o))
    }

    pub fn firsts(&self, _ctx: &Context) -> Vec<usize> {
        let mut indices = Vec::new();
        for (i, e) in self.states_dyn.iter().enumerate() {
            // println!("{} {e:?}", ctx.states_const[i].name);
            match (e.v, e.t, e.weak) {
                (State::True, State::True, _) => {
                    indices.push(i);
                    break;
                }
                (_, _, true) | (State::False, _, _) | (_, State::False, _) => {}
                _ => indices.push(i),
            }
        }
        indices
    }

    pub fn apply(&self, ctx: &Context, index: usize) -> Self {
        let mut states_dyn = self.states_dyn.clone();
        let state_const = &ctx.states_const[index];
        states_dyn[index].v = State::True;
        states_dyn[index].t = State::True;
        for state in &mut states_dyn[..index] {
            match (state.v, state.t) {
                (State::True, State::Unknown) => state.t = State::False,
                (State::Unknown, State::True) => state.v = State::False,
                (State::Unknown, State::Unknown) => state.weak = true,
                _ => {}
            }
        }
        for action in &state_const.actions {
            match action.change {
                Change::V(val) => states_dyn[action.index].v = val.into(),
                Change::T(val) => states_dyn[action.index].t = val.into(),
            }
        }
        Self {
            name: format!("{}-{}", self.name, state_const.name),
            finished: state_const.actions.is_empty(),
            states_dyn,
        }
    }

    pub fn next(&self, ctx: &Context) -> Vec<Node> {
        let firsts = self.firsts(ctx);
        println!("firsts: {firsts:?}");
        if firsts.len() == 1 {
            // fast forward trivial changes
            let next = self.apply(ctx, firsts[0]);
            if next.finished {
                return vec![next];
            } else {
                return next.next(ctx);
            }
        }
        firsts.into_iter().map(|i| self.apply(ctx, i)).collect()
    }
}
