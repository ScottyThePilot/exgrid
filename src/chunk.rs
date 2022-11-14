use std::ops::{Index, IndexMut};
use std::slice::Iter as ArrayIter;
use std::slice::IterMut as ArrayIterMut;
use std::vec::IntoIter as ArrayIntoIter;
use std::iter::{Enumerate, FilterMap, Flatten, FusedIterator};



#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChunkSparse<T, const S: usize> {
  inner: Chunk<Option<T>, S>
}

impl<T, const S: usize> ChunkSparse<T, S> {
  pub fn new() -> Self {
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

impl<T, const S: usize> Default for ChunkSparse<T, S> {
  #[inline]
  fn default() -> Self {
    ChunkSparse { inner: Chunk::default() }
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
  pub fn new() -> Self where T: Default {
    Self::default()
  }

  pub fn horizontal_slice(&self, y: usize) -> [T; S] where T: Clone {
    self.horizontal_slice_ref(y).clone()
  }

  pub fn horizontal_slice_ref(&self, y: usize) -> &[T; S] {
    assert!(y < S, "index out of bounds: the size is {S} but the y-index is {y}");
    &self.inner[y]
  }

  pub fn vertical_slice(&self, x: usize) -> [T; S] where T: Clone {
    assert!(x < S, "index out of bounds: the size is {S} but the x-index is {x}");
    array_init::array_init(|y| self.inner[y][x].clone())
  }

  pub fn vertical_slice_each_ref(&self, x: usize) -> [&T; S] {
    assert!(x < S, "index out of bounds: the size is {S} but the x-index is {x}");
    array_init::array_init(|y| &self.inner[y][x])
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
    let inner = Enumerate2::new(self.iter().inner);
    ChunkCells { inner }
  }

  pub fn cells_mut(&mut self) -> ChunkCellsMut<T, S> {
    let inner = Enumerate2::new(self.iter_mut().inner);
    ChunkCellsMut { inner }
  }

  pub fn into_cells(self) -> ChunkIntoCells<T, S> {
    let inner = Enumerate2::new(self.into_iter().inner);
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

impl<T: Default, const S: usize> Default for Chunk<T, S> {
  #[inline]
  fn default() -> Self {
    Chunk { inner: default_inner() }
  }
}

impl<'a, T, const S: usize> IntoIterator for &'a Chunk<T, S> {
  type Item = &'a T;
  type IntoIter = ChunkIter<'a, T, S>;

  fn into_iter(self) -> Self::IntoIter {
    let inner = cast_nested_array_ref(&self.inner).iter();
    ChunkIter { inner }
  }
}

impl<'a, T, const S: usize> IntoIterator for &'a mut Chunk<T, S> {
  type Item = &'a mut T;
  type IntoIter = ChunkIterMut<'a, T, S>;

  fn into_iter(self) -> Self::IntoIter {
    let inner = cast_nested_array_mut(&mut self.inner).iter_mut();
    ChunkIterMut { inner }
  }
}

impl<T, const S: usize> IntoIterator for Chunk<T, S> {
  type Item = T;
  type IntoIter = ChunkIntoIter<T, S>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    let inner = Vec::from(cast_nested_array(self.inner)).into_iter();
    ChunkIntoIter { inner }
  }
}

// This is necessary due to the array primitive's `Default` impl not actually being generic across all `N`.
fn default_inner<T: Default, const N: usize>() -> [[T; N]; N] {
  array_init::array_init(|_| {
    array_init::array_init(|_| {
      T::default()
    })
  })
}

fn cast_nested_array<T, const N: usize>(array: [[T; N]; N]) -> Box<[T]> {
  let Some(len) = usize::checked_mul(N, N) else { panic!() };
  let array: Box<[[T; N]; N]> = Box::new(array);
  unsafe {
    // Convert the box into a pointer, then a wide pointer, then a wide box-pointer
    let array_ptr = Box::into_raw(array) as *mut T;
    let ptr = std::slice::from_raw_parts_mut(array_ptr, len) as *mut [T];
    Box::from_raw(ptr)
  }
}

fn cast_nested_array_ref<T, const N: usize>(array: &[[T; N]; N]) -> &[T] {
  let Some(len) = usize::checked_mul(N, N) else { panic!() };
  unsafe {
    let ptr = array as *const [[T; N]; N] as *const T;
    std::slice::from_raw_parts(ptr, len)
  }
}

fn cast_nested_array_mut<T, const N: usize>(array: &mut [[T; N]; N]) -> &mut [T] {
  let Some(len) = usize::checked_mul(N, N) else { panic!() };
  unsafe {
    let ptr = array as *mut [[T; N]; N] as *mut T;
    std::slice::from_raw_parts_mut(ptr, len)
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
  inner: ArrayIter<'a, T>
}

impl_iterator_known_size!(ChunkIter, <'a, T, S>, &'a T, S * S);

#[repr(transparent)]
#[derive(Debug)]
pub struct ChunkIterMut<'a, T, const S: usize> {
  inner: ArrayIterMut<'a, T>
}

impl_iterator_known_size!(ChunkIterMut, <'a, T, S>, &'a mut T, S * S);

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ChunkIntoIter<T, const S: usize> {
  inner: ArrayIntoIter<T>
}

impl_iterator_known_size!(ChunkIntoIter, <T, S>, T, S * S);

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ChunkCells<'a, T, const S: usize> {
  inner: Enumerate2<ArrayIter<'a, T>, S>
}

impl_iterator_known_size!(ChunkCells, <'a, T, S>, ([usize; 2], &'a T), S * S);

#[repr(transparent)]
#[derive(Debug)]
pub struct ChunkCellsMut<'a, T, const S: usize> {
  inner: Enumerate2<ArrayIterMut<'a, T>, S>
}

impl_iterator_known_size!(ChunkCellsMut, <'a, T, S>, ([usize; 2], &'a mut T), S * S);

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ChunkIntoCells<T, const S: usize> {
  inner: Enumerate2<ArrayIntoIter<T>, S>
}

impl_iterator_known_size!(ChunkIntoCells, <T, S>, ([usize; 2], T), S * S);



macro_rules! map {
  ($S:expr, $expr:expr) => {
    match $expr {
      Some((i, item)) => {
        Some(([i % $S, i / $S], item))
      },
      None => None
    }
  };
}

/// Converts regular enumeration into 2D enumeration
#[derive(Debug, Clone)]
struct Enumerate2<I, const S: usize> {
  inner: Enumerate<I>
}

impl<I, const S: usize> Enumerate2<I, S> {
  pub fn new<T>(inner: I) -> Self where I: Iterator<Item = T> {
    Enumerate2 { inner: inner.enumerate() }
  }
}

impl<I, T, const S: usize> Iterator for Enumerate2<I, S>
where I: Iterator<Item = T> {
  type Item = ([usize; 2], T);

  fn next(&mut self) -> Option<Self::Item> {
    map!(S, self.inner.next())
  }

  #[inline]
  fn nth(&mut self, n: usize) -> Option<Self::Item> {
    map!(S, self.inner.nth(n))
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.inner.size_hint()
  }

  #[inline]
  fn fold<A, F>(self, init: A, mut f: F) -> A
  where F: FnMut(A, Self::Item) -> A {
    self.inner.fold(init, |a, (i, item)| {
      f(a, ([i % S, i / S], item))
    })
  }
}

impl<I, T, const S: usize> DoubleEndedIterator for Enumerate2<I, S>
where I: DoubleEndedIterator<Item = T> + ExactSizeIterator {
  #[inline]
  fn next_back(&mut self) -> Option<Self::Item> {
    map!(S, self.inner.next_back())
  }

  #[inline]
  fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
    map!(S, self.inner.nth_back(n))
  }

  #[inline]
  fn rfold<A, F>(self, init: A, mut f: F) -> A
  where F: FnMut(A, Self::Item) -> A {
    self.inner.rfold(init, |a, (i, item)| {
      f(a, ([i % S, i / S], item))
    })
  }
}

impl<I, const S: usize> FusedIterator for Enumerate2<I, S> where I: FusedIterator {}
