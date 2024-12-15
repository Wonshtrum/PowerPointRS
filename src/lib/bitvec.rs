use std::fmt;
use std::hash::Hash;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Deref, DerefMut, Not, Shl, Shr};

pub trait Bits:
    Copy
    + Sized
    + Default
    + PartialEq
    + Hash
    + BitAndAssign
    + BitOrAssign
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + Not<Output = Self>
    + Shr<usize, Output = Self>
    + Shl<usize, Output = Self>
    + fmt::Binary
    + fmt::Debug
{
    const SIZE: usize;
    const ZERO: Self;
    const ONE: Self;
    fn trailing_zeros(self) -> usize;
    fn get(self, n: usize) -> bool;
    fn set(&mut self, n: usize);
    fn unset(&mut self, n: usize);
}

pub struct BinaryFmt<T: Bits>(pub T);
impl<T: Bits> fmt::Debug for BinaryFmt<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:0width$b}", self.0, width = T::SIZE)
    }
}

macro_rules! impl_bits {
    ($T: ty) => {
        impl Bits for $T {
            const SIZE: usize = 8 * std::mem::size_of::<$T>();
            const ZERO: $T = 0;
            const ONE: $T = 0;
            fn trailing_zeros(self) -> usize {
                <$T>::trailing_zeros(self) as usize
            }
            fn get(self, n: usize) -> bool {
                self & (1 << n) != 0
            }
            fn set(&mut self, n: usize) {
                *self = *self | (1 << n);
            }
            fn unset(&mut self, n: usize) {
                *self = *self & !(1 << n);
            }
        }
    };
}

impl_bits!(u8);
impl_bits!(u16);
impl_bits!(u32);
impl_bits!(u64);
impl_bits!(usize);

#[derive(PartialEq, Eq, Hash)]
pub struct BitVec<T: Bits> {
    pub chunks: Box<[T]>,
}

impl<T: Bits> BitVec<T> {
    pub fn new(cap: usize) -> Self {
        println!("====================BitVec::new({cap})");
        Self {
            chunks: vec![T::ZERO; cap].into_boxed_slice(),
        }
    }
    pub fn get_chunk(&self, i: usize) -> T {
        unsafe { *self.chunks.get_unchecked(i) }
    }
    pub fn get_chunk_mut(&mut self, i: usize) -> &mut T {
        unsafe { self.chunks.get_unchecked_mut(i) }
    }
    pub fn get(&self, i: usize) -> bool {
        self.get_chunk(i / T::SIZE).get(i % T::SIZE)
    }
    pub fn set(&mut self, i: usize) {
        self.get_chunk_mut(i / T::SIZE).set(i % T::SIZE);
    }
    pub fn unset(&mut self, i: usize) {
        self.get_chunk_mut(i / T::SIZE).unset(i % T::SIZE);
    }
    pub fn clear(&mut self) {
        self.chunks.fill(T::ZERO);
    }
    pub fn copy(&mut self, src: &Self) {
        for (i, e) in src.chunks.iter().copied().enumerate() {
            *self.get_chunk_mut(i) = e;
        }
    }
    pub fn binop(&mut self, rhs: &Self, op: fn(&mut T, T)) {
        for (i, e) in rhs.chunks.iter().copied().enumerate() {
            op(self.get_chunk_mut(i), e);
        }
    }
}
impl<T: Bits> Clone for BitVec<T> {
    fn clone(&self) -> Self {
        println!("====================BitVec::clone({})", self.chunks.len());
        Self {
            chunks: self.chunks.clone(),
        }
    }
}
impl<T: Bits> Default for BitVec<T> {
    fn default() -> Self {
        Self {
            chunks: Box::new([]),
        }
    }
}
impl<T: Bits> Deref for BitVec<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        &self.chunks
    }
}
impl<T: Bits> DerefMut for BitVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.chunks
    }
}
impl<T: Bits> fmt::Debug for BitVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("[ ")?;
        for chunk in &self.chunks {
            write!(f, "{:?} ", &BinaryFmt(*chunk))?;
        }
        f.write_str("]")
    }
}
