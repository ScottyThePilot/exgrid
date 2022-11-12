use std::ops::{Index, IndexMut};
use std::slice::Iter as ArrayIter;
use std::slice::IterMut as ArrayIterMut;
use std::array::IntoIter as ArrayIntoIter;
use std::iter::{Enumerate, FilterMap, FlatMap, Flatten, FusedIterator};



#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChunkSparse<T, const S: usize> {
  inner: Chunk<Option<T>, S>
}

impl<T, const S: usize> ChunkSparse<T, S> {
  pub fn new() -> Self
  where [[Option<T>; S]; S]: Default {
    Self::default()
  }

  #[doc(hidden)]
  #[deprecated = "use `is_all_vacant` instead"]
  pub fn is_vacant(&self) -> bool {
    self.is_all_vacant()
  }

  /// Returns true if every cell in this chunk is `None`.
  pub fn is_all_vacant(&self) -> bool {
    self.inner.iter().all(|cell| cell.is_none())
  }

  /// Returns true if every cell in this chunk is `Some`.
  pub fn is_all_occupied(&self) -> bool {
    self.inner.iter().all(|cell| cell.is_some())
  }

  pub fn horizontal_slice(&self, y: usize) -> [Option<T>; S]
  where T: Copy {
    self.inner.horizontal_slice(y)
  }

  pub fn vertical_slice(&self, x: usize) -> [Option<T>; S]
  where T: Copy {
    self.inner.vertical_slice(x)
  }

  #[inline]
  pub fn iter(&self) -> ChunkSparseIter<T, S> {
    self.into_iter()
  }

  #[inline]
  pub fn iter_mut(&mut self) -> ChunkSparseIterMut<T, S> {
    self.into_iter()
  }

  pub fn cells(&self) -> ChunkSparseCells<T, S> {
    let f: fn(([usize; 2], &Option<T>)) -> Option<([usize; 2], &T)> = |(i, v)| v.as_ref().map(|v| (i, v));
    let inner = self.inner.cells().filter_map(f);
    ChunkSparseCells { inner }
  }

  pub fn cells_mut(&mut self) -> ChunkSparseCellsMut<T, S> {
    let f: fn(([usize; 2], &mut Option<T>)) -> Option<([usize; 2], &mut T)> = |(i, v)| v.as_mut().map(|v| (i, v));
    let inner = self.inner.cells_mut().filter_map(f);
    ChunkSparseCellsMut { inner }
  }

  pub fn into_cells(self) -> ChunkSparseIntoCells<T, S> {
    let f: fn(([usize; 2], Option<T>)) -> Option<([usize; 2], T)> = |(i, v)| v.map(|v| (i, v));
    let inner = self.inner.into_cells().filter_map(f);
    ChunkSparseIntoCells { inner }
  }
}

impl<T, const S: usize> Index<[usize; 2]> for ChunkSparse<T, S> {
  type Output = Option<T>;

  #[inline]
  fn index(&self, pos: [usize; 2]) -> &Option<T> {
    &self.inner[pos]
  }
}

impl<T, const S: usize> IndexMut<[usize; 2]> for ChunkSparse<T, S> {
  #[inline]
  fn index_mut(&mut self, pos: [usize; 2]) -> &mut Option<T> {
    &mut self.inner[pos]
  }
}

impl<T, const S: usize> Default for ChunkSparse<T, S>
where [[Option<T>; S]; S]: Default {
  #[inline]
  fn default() -> Self {
    ChunkSparse { inner: Default::default() }
  }
}

impl<'a, T, const S: usize> IntoIterator for &'a ChunkSparse<T, S> {
  type Item = &'a T;
  type IntoIter = ChunkSparseIter<'a, T, S>;

  fn into_iter(self) -> Self::IntoIter {
    let inner = (&self.inner).into_iter().flatten();
    ChunkSparseIter { inner }
  }
}

impl<'a, T, const S: usize> IntoIterator for &'a mut ChunkSparse<T, S> {
  type Item = &'a mut T;
  type IntoIter = ChunkSparseIterMut<'a, T, S>;

  fn into_iter(self) -> Self::IntoIter {
    let inner = (&mut self.inner).into_iter().flatten();
    ChunkSparseIterMut { inner }
  }
}

impl<T, const S: usize> IntoIterator for ChunkSparse<T, S> {
  type Item = T;
  type IntoIter = ChunkSparseIntoIter<T, S>;

  fn into_iter(self) -> Self::IntoIter {
    let inner = self.inner.into_iter().flatten();
    ChunkSparseIntoIter { inner }
  }
}


#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Chunk<T, const S: usize> {
  inner: [[T; S]; S]
}

impl<T, const S: usize> Chunk<T, S> {
  pub fn new() -> Self
  where [[T; S]; S]: Default {
    Self::default()
  }

  pub fn horizontal_slice(&self, y: usize) -> [T; S]
  where T: Copy {
    assert!(y < S, "index out of bounds: the size is {S} but the y-index is {y}");
    self.inner[y]
  }

  pub fn vertical_slice(&self, x: usize) -> [T; S]
  where T: Copy {
    assert!(x < S, "index out of bounds: the size is {S} but the x-index is {x}");
    self.inner.map(|s| s[x])
  }

  #[inline]
  pub fn iter(&self) -> ChunkIter<T, S> {
    self.into_iter()
  }

  #[inline]
  pub fn iter_mut(&mut self) -> ChunkIterMut<T, S> {
    self.into_iter()
  }

  pub fn cells(&self) -> ChunkCells<T, S> {
    let f: fn((usize, &[T; S])) -> EnumerateExtra<ArrayIter<T>> = |i| i.into();
    let inner = (&self.inner).into_iter().enumerate().flat_map(f);
    ChunkCells { inner }
  }

  pub fn cells_mut(&mut self) -> ChunkCellsMut<T, S> {
    let f: fn((usize, &mut [T; S])) -> EnumerateExtra<ArrayIterMut<T>> = |i| i.into();
    let inner = (&mut self.inner).into_iter().enumerate().flat_map(f);
    ChunkCellsMut { inner }
  }

  pub fn into_cells(self) -> ChunkIntoCells<T, S> {
    let inner = (self.inner).into_iter().enumerate()
      .flat_map(EnumerateExtra::from as _);
    ChunkIntoCells { inner }
  }
}

impl<T, const S: usize> Index<[usize; 2]> for Chunk<T, S> {
  type Output = T;

  #[inline]
  fn index(&self, [x, y]: [usize; 2]) -> &T {
    &self.inner[y][x]
  }
}

impl<T, const S: usize> IndexMut<[usize; 2]> for Chunk<T, S> {
  #[inline]
  fn index_mut(&mut self, [x, y]: [usize; 2]) -> &mut T {
    &mut self.inner[y][x]
  }
}

impl<T, const S: usize> Default for Chunk<T, S>
where [[T; S]; S]: Default {
  #[inline]
  fn default() -> Self {
    Chunk { inner: Default::default() }
  }
}

impl<'a, T, const S: usize> IntoIterator for &'a Chunk<T, S> {
  type Item = &'a T;
  type IntoIter = ChunkIter<'a, T, S>;

  fn into_iter(self) -> Self::IntoIter {
    let f: fn(&[_; S]) -> ArrayIter<_> = |i| i.into_iter();
    let inner = (&self.inner).into_iter().flat_map(f);
    ChunkIter { inner }
  }
}

impl<'a, T, const S: usize> IntoIterator for &'a mut Chunk<T, S> {
  type Item = &'a mut T;
  type IntoIter = ChunkIterMut<'a, T, S>;

  fn into_iter(self) -> Self::IntoIter {
    let f: fn(&mut [_; S]) -> ArrayIterMut<_> = |i| i.into_iter();
    let inner = (&mut self.inner).into_iter().flat_map(f);
    ChunkIterMut { inner }
  }
}

impl<T, const S: usize> IntoIterator for Chunk<T, S> {
  type Item = T;
  type IntoIter = ChunkIntoIter<T, S>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    let inner = self.inner.into_iter()
      .flat_map(<[T; S]>::into_iter as _);
    ChunkIntoIter { inner }
  }
}



#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ChunkSparseIter<'a, T, const S: usize> {
  inner: Flatten<ChunkIter<'a, Option<T>, S>>
}

impl_iterator!(ChunkSparseIter, <'a, T, S>, &'a T, (0, Some(S * S)));

#[repr(transparent)]
#[derive(Debug)]
pub struct ChunkSparseIterMut<'a, T, const S: usize> {
  inner: Flatten<ChunkIterMut<'a, Option<T>, S>>
}

impl_iterator!(ChunkSparseIterMut, <'a, T, S>, &'a mut T, (0, Some(S * S)));

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ChunkSparseIntoIter<T, const S: usize> {
  inner: Flatten<ChunkIntoIter<Option<T>, S>>
}

impl_iterator!(ChunkSparseIntoIter, <T, S>, T, (0, Some(S * S)));

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ChunkSparseCells<'a, T, const S: usize> {
  inner: FilterMap<
    ChunkCells<'a, Option<T>, S>,
    fn(([usize; 2], &Option<T>)) -> Option<([usize; 2], &T)>
  >
}

impl_iterator!(ChunkSparseCells, <'a, T, S>, ([usize; 2], &'a T), (0, Some(S * S)));

#[repr(transparent)]
#[derive(Debug)]
pub struct ChunkSparseCellsMut<'a, T, const S: usize> {
  inner: FilterMap<
    ChunkCellsMut<'a, Option<T>, S>,
    fn(([usize; 2], &mut Option<T>)) -> Option<([usize; 2], &mut T)>
  >
}

impl_iterator!(ChunkSparseCellsMut, <'a, T, S>, ([usize; 2], &'a mut T), (0, Some(S * S)));

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ChunkSparseIntoCells<T, const S: usize> {
  inner: FilterMap<
    ChunkIntoCells<Option<T>, S>,
    fn(([usize; 2], Option<T>)) -> Option<([usize; 2], T)>
  >
}

impl_iterator!(ChunkSparseIntoCells, <T, S>, ([usize; 2], T), (0, Some(S * S)));



#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ChunkIter<'a, T, const S: usize> {
  inner: FlatMap<
    ArrayIter<'a, [T; S]>,
    ArrayIter<'a, T>,
    fn(&[T; S]) -> ArrayIter<T>
  >
}

impl_iterator_known_size!(ChunkIter, <'a, T, S>, &'a T, S * S);

#[repr(transparent)]
#[derive(Debug)]
pub struct ChunkIterMut<'a, T, const S: usize> {
  inner: FlatMap<
    ArrayIterMut<'a, [T; S]>,
    ArrayIterMut<'a, T>,
    fn(&mut [T; S]) -> ArrayIterMut<T>
  >
}

impl_iterator_known_size!(ChunkIterMut, <'a, T, S>, &'a mut T, S * S);

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ChunkIntoIter<T, const S: usize> {
  inner: FlatMap<
    ArrayIntoIter<[T; S], S>,
    ArrayIntoIter<T, S>,
    fn([T; S]) -> ArrayIntoIter<T, S>
  >
}

impl_iterator_known_size!(ChunkIntoIter, <T, S>, T, S * S);

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ChunkCells<'a, T, const S: usize> {
  inner: FlatMap<
    Enumerate<ArrayIter<'a, [T; S]>>,
    EnumerateExtra<ArrayIter<'a, T>>,
    fn((usize, &[T; S])) -> EnumerateExtra<ArrayIter<T>>
  >
}

impl_iterator_known_size!(ChunkCells, <'a, T, S>, ([usize; 2], &'a T), S * S);

#[repr(transparent)]
#[derive(Debug)]
pub struct ChunkCellsMut<'a, T, const S: usize> {
  inner: FlatMap<
    Enumerate<ArrayIterMut<'a, [T; S]>>,
    EnumerateExtra<ArrayIterMut<'a, T>>,
    fn((usize, &mut [T; S])) -> EnumerateExtra<ArrayIterMut<T>>
  >
}

impl_iterator_known_size!(ChunkCellsMut, <'a, T, S>, ([usize; 2], &'a mut T), S * S);

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ChunkIntoCells<T, const S: usize> {
  inner: FlatMap<
    Enumerate<ArrayIntoIter<[T; S], S>>,
    EnumerateExtra<ArrayIntoIter<T, S>>,
    fn((usize, [T; S])) -> EnumerateExtra<ArrayIntoIter<T, S>>
  >
}

impl_iterator_known_size!(ChunkIntoCells, <T, S>, ([usize; 2], T), S * S);



macro_rules! map {
  ($self:ident, $expr:expr) => {
    match $expr {
      Some((other_index, value)) => {
        Some(([other_index, $self.index], value))
      },
      None => None
    }
  };
}

#[derive(Debug, Clone)]
struct EnumerateExtra<I> {
  index: usize,
  iter: Enumerate<I>
}

impl<I> EnumerateExtra<I> {
  pub fn new<T>(index: usize, iter: I) -> Self
  where I: Iterator<Item = T> {
    EnumerateExtra { index, iter: iter.enumerate() }
  }
}

impl<I, T> From<(usize, I)> for EnumerateExtra<I::IntoIter>
where I: IntoIterator<Item = T> {
  #[inline]
  fn from((index, iter): (usize, I)) -> Self {
    Self::new::<T>(index, iter.into_iter())
  }
}

impl<I, T> Iterator for EnumerateExtra<I>
where I: Iterator<Item = T> {
  type Item = ([usize; 2], T);

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    map!(self, self.iter.next())
  }

  #[inline]
  fn nth(&mut self, n: usize) -> Option<Self::Item> {
    map!(self, self.iter.nth(n))
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.iter.size_hint()
  }

  #[inline]
  fn fold<B, F>(self, init: B, f: F) -> B
  where F: FnMut(B, Self::Item) -> B, {
    self.iter.fold(init, wrap_fold_fn(self.index, f))
  }
}

impl<I, T> DoubleEndedIterator for EnumerateExtra<I>
where I: DoubleEndedIterator<Item = T> + ExactSizeIterator {
  #[inline]
  fn next_back(&mut self) -> Option<Self::Item> {
    map!(self, self.iter.next_back())
  }

  #[inline]
  fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
    map!(self, self.iter.nth_back(n))
  }

  #[inline]
  fn rfold<A, F>(self, init: A, f: F) -> A
  where F: FnMut(A, Self::Item) -> A {
    self.iter.rfold(init, wrap_fold_fn(self.index, f))
  }
}

impl<I> FusedIterator for EnumerateExtra<I> where I: FusedIterator {}



fn wrap_fold_fn<T, A, F>(index: usize, mut f: F) -> impl FnMut(A, (usize, T)) -> A
where F: FnMut(A, ([usize; 2], T)) -> A {
  move |acc, (other_index, value)| {
    f(acc, ([index, other_index], value))
  }
}
