use super::{Chunk, ChunkSparse};
use crate::LocalPos;

use std::iter::{Enumerate, FilterMap, Flatten, FusedIterator};



/// An iterator over all of the occupied cells in a sparse chunk.
/// Yields a reference to the cell's value.
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ChunkSparseIter<'a, T, const S: usize> {
  inner: Flatten<ChunkIter<'a, Option<T>, S>>
}

impl<'a, T, const S: usize> ChunkSparseIter<'a, T, S> {
  pub(crate) fn new(chunk: &'a ChunkSparse<T, S>) -> Self {
    let inner = ChunkIter::new(&chunk.inner).flatten();
    ChunkSparseIter { inner }
  }
}

impl_iterator!(ChunkSparseIter, <'a, T, S>, &'a T, (0, Some(S * S)));

/// An iterator over all of the occupied cells in a sparse chunk.
/// Yields a mutable reference to the cell's value.
#[repr(transparent)]
#[derive(Debug)]
pub struct ChunkSparseIterMut<'a, T, const S: usize> {
  inner: Flatten<ChunkIterMut<'a, Option<T>, S>>
}

impl<'a, T, const S: usize> ChunkSparseIterMut<'a, T, S> {
  pub(crate) fn new(chunk: &'a mut ChunkSparse<T, S>) -> Self {
    let inner = ChunkIterMut::new(&mut chunk.inner).flatten();
    ChunkSparseIterMut { inner }
  }
}

impl_iterator!(ChunkSparseIterMut, <'a, T, S>, &'a mut T, (0, Some(S * S)));

/// An iterator over all of the occupied cells in a sparse chunk.
/// Yields the cell's value.
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ChunkSparseIntoIter<T, const S: usize> {
  inner: Flatten<ChunkIntoIter<Option<T>, S>>
}

impl<T, const S: usize> ChunkSparseIntoIter<T, S> {
  pub(crate) fn new(chunk: ChunkSparse<T, S>) -> Self {
    let inner = ChunkIntoIter::new(chunk.inner).flatten();
    ChunkSparseIntoIter { inner }
  }
}

impl_iterator!(ChunkSparseIntoIter, <T, S>, T, (0, Some(S * S)));



/// An 'enumerating' iterator over all of the occupied cells in a sparse chunk.
/// Yields the position of the cell along with a reference to the cell's value.
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ChunkSparseCells<'a, T, const S: usize> {
  inner: FilterMap<ChunkCells<'a, Option<T>, S>, super::FilterCells<T, S>>
}

impl<'a, T, const S: usize> ChunkSparseCells<'a, T, S> {
  pub(crate) fn new(chunk: &'a ChunkSparse<T, S>) -> Self {
    let inner = ChunkCells::new(&chunk.inner)
      .filter_map(ChunkSparse::<T, S>::NEW_CELLS);
    ChunkSparseCells { inner }
  }
}

impl_iterator!(ChunkSparseCells, <'a, T, S>, (LocalPos, &'a T), (0, Some(S * S)));

/// An 'enumerating' iterator over all of the occupied cells in a sparse chunk.
/// Yields the position of the cell along with a mutable reference to the cell's value.
#[repr(transparent)]
#[derive(Debug)]
pub struct ChunkSparseCellsMut<'a, T, const S: usize> {
  inner: FilterMap<ChunkCellsMut<'a, Option<T>, S>, super::FilterCellsMut<T, S>>
}

impl<'a, T, const S: usize> ChunkSparseCellsMut<'a, T, S> {
  pub(crate) fn new(chunk: &'a mut ChunkSparse<T, S>) -> Self {
    let inner = ChunkCellsMut::new(&mut chunk.inner)
      .filter_map(ChunkSparse::<T, S>::NEW_CELLS_MUT);
    ChunkSparseCellsMut { inner }
  }
}

impl_iterator!(ChunkSparseCellsMut, <'a, T, S>, (LocalPos, &'a mut T), (0, Some(S * S)));

/// An 'enumerating' iterator over all of the occupied cells in a sparse chunk.
/// Yields the position of the cell along with the cell's value.
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ChunkSparseIntoCells<T, const S: usize> {
  inner: FilterMap<ChunkIntoCells<Option<T>, S>, super::FilterIntoCells<T, S>>
}

impl<T, const S: usize> ChunkSparseIntoCells<T, S> {
  pub(crate) fn new(chunk: ChunkSparse<T, S>) -> Self {
    let inner = ChunkIntoCells::new(chunk.inner)
      .filter_map(ChunkSparse::<T, S>::NEW_INTO_CELLS);
    ChunkSparseIntoCells { inner }
  }
}

impl_iterator!(ChunkSparseIntoCells, <T, S>, (LocalPos, T), (0, Some(S * S)));



/// An iterator over all of the cells in a chunk.
/// Yields a reference to the cell's value.
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ChunkIter<'a, T, const S: usize> {
  inner: <&'a [T] as IntoIterator>::IntoIter
}

impl<'a, T, const S: usize> ChunkIter<'a, T, S> {
  pub(crate) fn new(chunk: &'a Chunk<T, S>) -> Self {
    let inner = super::cast_nested_array_ref(&chunk.inner).iter();
    ChunkIter { inner }
  }
}

impl_iterator_known_size!(ChunkIter, <'a, T, S>, &'a T, S * S);

/// An iterator over all of the cells in a chunk.
/// Yields a mutable reference to the cell's value.
#[repr(transparent)]
#[derive(Debug)]
pub struct ChunkIterMut<'a, T, const S: usize> {
  inner: <&'a mut [T] as IntoIterator>::IntoIter
}

impl<'a, T, const S: usize> ChunkIterMut<'a, T, S> {
  pub(crate) fn new(chunk: &'a mut Chunk<T, S>) -> Self {
    let inner = super::cast_nested_array_mut(&mut chunk.inner).iter_mut();
    ChunkIterMut { inner }
  }
}

impl_iterator_known_size!(ChunkIterMut, <'a, T, S>, &'a mut T, S * S);

/// An iterator over all of the cells in a chunk.
/// Yields the cell's value.
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ChunkIntoIter<T, const S: usize> {
  inner: <Vec<T> as IntoIterator>::IntoIter
}

impl<T, const S: usize> ChunkIntoIter<T, S> {
  pub(crate) fn new(chunk: Chunk<T, S>) -> Self {
    let inner = Vec::from(super::cast_nested_array(chunk.inner)).into_iter();
    ChunkIntoIter { inner }
  }
}

impl_iterator_known_size!(ChunkIntoIter, <T, S>, T, S * S);



/// An 'enumerating' iterator over all of the cells in a chunk.
/// Yields the position of the cell along with a reference to the cell's value.
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ChunkCells<'a, T, const S: usize> {
  inner: Enumerate2<ChunkIter<'a, T, S>, S>
}

impl<'a, T, const S: usize> ChunkCells<'a, T, S> {
  pub(crate) fn new(chunk: &'a Chunk<T, S>) -> Self {
    let inner = Enumerate2::new(ChunkIter::new(chunk));
    ChunkCells { inner }
  }
}

impl_iterator_known_size!(ChunkCells, <'a, T, S>, (LocalPos, &'a T), S * S);

/// An 'enumerating' iterator over all of the cells in a chunk.
/// Yields the position of the cell along with a mutable reference to the cell's value.
#[repr(transparent)]
#[derive(Debug)]
pub struct ChunkCellsMut<'a, T, const S: usize> {
  inner: Enumerate2<ChunkIterMut<'a, T, S>, S>
}

impl<'a, T, const S: usize> ChunkCellsMut<'a, T, S> {
  pub(crate) fn new(chunk: &'a mut Chunk<T, S>) -> Self {
    let inner = Enumerate2::new(ChunkIterMut::new(chunk));
    ChunkCellsMut { inner }
  }
}

impl_iterator_known_size!(ChunkCellsMut, <'a, T, S>, (LocalPos, &'a mut T), S * S);

/// An 'enumerating' iterator over all of the cells in a chunk.
/// Yields the position of the cell along with the cell's value.
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ChunkIntoCells<T, const S: usize> {
  inner: Enumerate2<ChunkIntoIter<T, S>, S>
}

impl<T, const S: usize> ChunkIntoCells<T, S> {
  pub(crate) fn new(chunk: Chunk<T, S>) -> Self {
    let inner = Enumerate2::new(ChunkIntoIter::new(chunk));
    ChunkIntoCells { inner }
  }
}

impl_iterator_known_size!(ChunkIntoCells, <T, S>, (LocalPos, T), S * S);



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

/// Converts regular enumeration into 2D enumeration.
#[derive(Debug, Clone)]
pub(crate) struct Enumerate2<I, const S: usize> {
  inner: Enumerate<I>
}

impl<I, const S: usize> Enumerate2<I, S> {
  pub(crate) fn new<T>(inner: I) -> Self where I: Iterator<Item = T> {
    Enumerate2 { inner: inner.enumerate() }
  }
}

impl<I, T, const S: usize> Iterator for Enumerate2<I, S>
where I: Iterator<Item = T> {
  type Item = (LocalPos, T);

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
