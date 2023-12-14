pub type Cell = usize;
pub const CELL_SIZE: usize = std::mem::size_of::<Cell>() * 8 as usize;
pub const CELL_SHIFT: usize = CELL_SIZE.trailing_zeros() as usize;
pub const CELL_MASK: usize = CELL_SIZE - 1;

#[derive(Clone, Debug)]
pub struct Filter {
    cells: Vec<Cell>,
}
impl Filter {
    pub fn new(n: usize) -> Self {
        let capacity = (n + CELL_SIZE - 1) / CELL_SIZE;
        Self {
            cells: vec![0; capacity],
        }
    }

    #[inline(always)]
    pub fn get(&self, n: usize) -> bool {
        let index = n >> CELL_SHIFT;
        self.cells[index] & (1 << (n & CELL_MASK)) != 0
    }
    #[inline(always)]
    pub fn set(&mut self, n: usize) {
        let index = n >> CELL_SHIFT;
        self.cells[index] |= 1 << (n & CELL_MASK);
    }
    #[inline(always)]
    pub fn unset(&mut self, n: usize) {
        let index = n >> CELL_SHIFT;
        self.cells[index] &= !(1 << (n & CELL_MASK));
    }
    #[inline(always)]
    pub fn toggle(&mut self, n: usize) {
        let index = n >> CELL_SHIFT;
        self.cells[index] ^= 1 << (n & CELL_MASK);
    }

    pub fn last(&self) -> Option<usize> {
        for (index, val) in self.cells.iter().enumerate().rev() {
            if *val != 0 {
                return Some((index << CELL_SHIFT) + CELL_SIZE - 1 - val.leading_zeros() as usize);
            }
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct DoubleFilter {
    pub cells: Vec<(Cell, Cell)>,
}
impl DoubleFilter {
    pub fn new(n: usize) -> Self {
        let capacity = (n + CELL_SIZE - 1) / CELL_SIZE;
        Self {
            cells: vec![(0, 0); capacity],
        }
    }

    #[inline(always)]
    pub fn get(&self, n: usize) -> (bool, bool) {
        let index = n >> CELL_SHIFT;
        let sub_index = 1 << (n & CELL_MASK);
        let cell = &self.cells[index];
        (cell.0 & sub_index != 0, cell.1 & sub_index != 0)
    }
    #[inline(always)]
    pub fn set(&mut self, n: usize, first: bool, second: bool) {
        let index = n >> CELL_SHIFT;
        let set_mask = 1 << (n & CELL_MASK);
        let unset_mask = !set_mask;
        let cell = &mut self.cells[index];
        if first {
            cell.0 |= set_mask;
        } else {
            cell.0 &= unset_mask;
        }
        if second {
            cell.1 |= set_mask;
        } else {
            cell.1 &= unset_mask;
        }
    }

    pub fn last(&self) -> Option<usize> {
        for (index, val) in self.cells.iter().enumerate().rev() {
            let combined = val.0 & val.1;
            if combined != 0 {
                return Some(
                    (index << CELL_SHIFT) + CELL_SIZE - 1 - combined.leading_zeros() as usize,
                );
            }
        }
        None
    }
}
