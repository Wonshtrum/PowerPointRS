type CELL = usize;
const CELL_SIZE: usize = std::mem::size_of::<CELL>() * 8 as usize;
const CELL_SHIFT: usize = CELL_SIZE.trailing_zeros() as usize;
const CELL_MASK: usize = CELL_SIZE - 1;
#[derive(Clone, Debug)]
pub struct Filter {
    cells: Vec<CELL>,
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

    pub fn first(&self) -> Option<usize> {
        for (index, val) in self.cells.iter().enumerate() {
            if *val != 0 {
                return Some(index << CELL_SHIFT);
            }
        }
        None
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
