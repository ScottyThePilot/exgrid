use super::{ExGrid, ExGridSparse};
use crate::{GlobalPos, ChunkPos, LocalPos};
use crate::chunk::*;

use std::collections::hash_map::{
  Iter as HashMapIter,
  IterMut as HashMapIterMut,
  IntoIter as HashMapIntoIter,
  Values as HashMapValues,
  ValuesMut as HashMapValuesMut,
  IntoValues as HashMapIntoValues
};
use std::iter::{Flatten, FlatMap, FusedIterator};

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ExGridSparseIter<'a, T, const S: usize> {
  inner: Flatten<HashMapValues<'a, ChunkPos, ChunkSparse<T, S>>>
}

impl<'a, T, const S: usize> ExGridSparseIter<'a, T, S> {
  pub(crate) fn new<H>(grid: &'a ExGridSparse<T, S, H>) -> Self {
    let inner = grid.chunks.values().flatten();
    ExGridSparseIter { inner }
  }
}

impl_iterator_no_double_ended!(ExGridSparseIter, <'a, T, S>, &'a T);

#[repr(transparent)]
#[derive(Debug)]
pub struct ExGridSparseIterMut<'a, T, const S: usize> {
  inner: Flatten<HashMapValuesMut<'a, ChunkPos, ChunkSparse<T, S>>>
}

impl<'a, T, const S: usize> ExGridSparseIterMut<'a, T, S> {
  pub(crate) fn new<H>(grid: &'a mut ExGridSparse<T, S, H>) -> Self {
    let inner = grid.chunks.values_mut().flatten();
    ExGridSparseIterMut { inner }
  }
}

impl_iterator_no_double_ended!(ExGridSparseIterMut, <'a, T, S>, &'a mut T);

#[repr(transparent)]
#[derive(Debug)]
pub struct ExGridSparseIntoIter<T, const S: usize> {
  inner: Flatten<HashMapIntoValues<ChunkPos, ChunkSparse<T, S>>>
}

impl<T, const S: usize> ExGridSparseIntoIter<T, S> {
  pub(crate) fn new<H>(grid: ExGridSparse<T, S, H>) -> Self {
    let inner = grid.chunks.into_values().flatten();
    ExGridSparseIntoIter { inner }
  }
}

impl_iterator_no_double_ended!(ExGridSparseIntoIter, <T, S>, T);

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ExGridSparseCells<'a, T, const S: usize> {
  inner: FlatMap<
    HashMapIter<'a, ChunkPos, ChunkSparse<T, S>>,
    Compose<ChunkSparseCells<'a, T, S>, S>,
    super::FilterSparseCells<T, S>
  >
}

impl<'a, T, const S: usize> ExGridSparseCells<'a, T, S> {
  pub(crate) fn new<H>(grid: &'a ExGridSparse<T, S, H>) -> Self {
    let inner = grid.chunks.iter().flat_map(ExGridSparse::<T, S, H>::NEW_SPARSE_CELLS);
    ExGridSparseCells { inner }
  }
}

impl_iterator_no_double_ended!(ExGridSparseCells, <'a, T, S>, (GlobalPos, &'a T));

#[repr(transparent)]
#[derive(Debug)]
pub struct ExGridSparseCellsMut<'a, T, const S: usize> {
  inner: FlatMap<
    HashMapIterMut<'a, ChunkPos, ChunkSparse<T, S>>,
    Compose<ChunkSparseCellsMut<'a, T, S>, S>,
    super::FilterSparseCellsMut<T, S>
  >
}

impl<'a, T, const S: usize> ExGridSparseCellsMut<'a, T, S> {
  pub(crate) fn new<H>(grid: &'a mut ExGridSparse<T, S, H>) -> Self {
    let inner = grid.chunks.iter_mut().flat_map(ExGridSparse::<T, S, H>::NEW_SPARSE_CELLS_MUT);
    ExGridSparseCellsMut { inner }
  }
}

impl_iterator_no_double_ended!(ExGridSparseCellsMut, <'a, T, S>, (GlobalPos, &'a mut T));

#[repr(transparent)]
#[derive(Debug)]
pub struct ExGridSparseIntoCells<T, const S: usize> {
  inner: FlatMap<
    HashMapIntoIter<ChunkPos, ChunkSparse<T, S>>,
    Compose<ChunkSparseIntoCells<T, S>, S>,
    super::FilterSparseIntoCells<T, S>
  >
}

impl<T, const S: usize> ExGridSparseIntoCells<T, S> {
  pub(crate) fn new<H>(grid: ExGridSparse<T, S, H>) -> Self {
    let inner = grid.chunks.into_iter().flat_map(ExGridSparse::<T, S, H>::NEW_SPARSE_INTO_CELLS);
    ExGridSparseIntoCells { inner }
  }
}

impl_iterator_no_double_ended!(ExGridSparseIntoCells, <T, S>, (GlobalPos, T));



#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ExGridIter<'a, T, const S: usize> {
  inner: Flatten<HashMapValues<'a, ChunkPos, Chunk<T, S>>>
}

impl<'a, T, const S: usize> ExGridIter<'a, T, S> {
  pub(crate) fn new<H>(grid: &'a ExGrid<T, S, H>) -> Self {
    let inner = grid.chunks.values().flatten();
    ExGridIter { inner }
  }
}

impl_iterator_no_double_ended!(ExGridIter, <'a, T, S>, &'a T);

#[repr(transparent)]
#[derive(Debug)]
pub struct ExGridIterMut<'a, T, const S: usize> {
  inner: Flatten<HashMapValuesMut<'a, ChunkPos, Chunk<T, S>>>
}

impl<'a, T, const S: usize> ExGridIterMut<'a, T, S> {
  pub(crate) fn new<H>(grid: &'a mut ExGrid<T, S, H>) -> Self {
    let inner = grid.chunks.values_mut().flatten();
    ExGridIterMut { inner }
  }
}

impl_iterator_no_double_ended!(ExGridIterMut, <'a, T, S>, &'a mut T);

#[repr(transparent)]
#[derive(Debug)]
pub struct ExGridIntoIter<T, const S: usize> {
  inner: Flatten<HashMapIntoValues<ChunkPos, Chunk<T, S>>>
}

impl<T, const S: usize> ExGridIntoIter<T, S> {
  pub(crate) fn new<H>(grid: ExGrid<T, S, H>) -> Self {
    let inner = grid.chunks.into_values().flatten();
    ExGridIntoIter { inner }
  }
}

impl_iterator_no_double_ended!(ExGridIntoIter, <T, S>, T);

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ExGridCells<'a, T, const S: usize> {
  inner: FlatMap<
    HashMapIter<'a, ChunkPos, Chunk<T, S>>,
    Compose<ChunkCells<'a, T, S>, S>,
    super::FilterCells<T, S>
  >
}

impl<'a, T, const S: usize> ExGridCells<'a, T, S> {
  pub(crate) fn new<H>(grid: &'a ExGrid<T, S, H>) -> Self {
    let inner = grid.chunks.iter().flat_map(ExGrid::<T, S, H>::NEW_CELLS);
    ExGridCells { inner }
  }
}

impl_iterator_no_double_ended!(ExGridCells, <'a, T, S>, (GlobalPos, &'a T));

#[repr(transparent)]
#[derive(Debug)]
pub struct ExGridCellsMut<'a, T, const S: usize> {
  inner: FlatMap<
    HashMapIterMut<'a, ChunkPos, Chunk<T, S>>,
    Compose<ChunkCellsMut<'a, T, S>, S>,
    super::FilterCellsMut<T, S>
  >
}

impl<'a, T, const S: usize> ExGridCellsMut<'a, T, S> {
  pub(crate) fn new<H>(grid: &'a mut ExGrid<T, S, H>) -> Self {
    let inner = grid.chunks.iter_mut().flat_map(ExGrid::<T, S, H>::NEW_CELLS_MUT);
    ExGridCellsMut { inner }
  }
}

impl_iterator_no_double_ended!(ExGridCellsMut, <'a, T, S>, (GlobalPos, &'a mut T));

#[repr(transparent)]
#[derive(Debug)]
pub struct ExGridIntoCells<T, const S: usize> {
  inner: FlatMap<
    HashMapIntoIter<ChunkPos, Chunk<T, S>>,
    Compose<ChunkIntoCells<T, S>, S>,
    super::FilterIntoCells<T, S>
  >
}

impl<T, const S: usize> ExGridIntoCells<T, S> {
  pub(crate) fn new<H>(grid: ExGrid<T, S, H>) -> Self {
    let inner = grid.chunks.into_iter().flat_map(ExGrid::<T, S, H>::NEW_INTO_CELLS);
    ExGridIntoCells { inner }
  }
}

impl_iterator_no_double_ended!(ExGridIntoCells, <T, S>, (GlobalPos, T));



macro_rules! map {
  ($chunk:expr, $expr:expr, $S:expr) => {
    match $expr {
      Some((local, value)) => {
        Some((super::compose::<$S>($chunk, local), value))
      },
      None => None
    }
  };
}

#[derive(Debug, Clone)]
pub(crate) struct Compose<I, const S: usize> {
  chunk: ChunkPos,
  cells: I
}

impl<I, const S: usize> Compose<I, S> {
  pub fn new<T>(chunk: ChunkPos, cells: I) -> Self
  where I: Iterator<Item = (LocalPos, T)> {
    Compose { chunk, cells }
  }
}

impl<T, I, const S: usize> Iterator for Compose<I, S>
where I: Iterator<Item = (LocalPos, T)> {
  type Item = (GlobalPos, T);

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    map!(self.chunk, self.cells.next(), S)
  }

  #[inline]
  fn nth(&mut self, n: usize) -> Option<Self::Item> {
    map!(self.chunk, self.cells.nth(n), S)
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.cells.size_hint()
  }

  #[inline]
  fn fold<A, F>(self, init: A, mut f: F) -> A
  where F: FnMut(A, Self::Item) -> A, {
    self.cells.fold(init, move |acc, (local, value)| {
      f(acc, (super::compose::<S>(self.chunk, local), value))
    })
  }
}

impl<T, I, const S: usize> DoubleEndedIterator for Compose<I, S>
where I: DoubleEndedIterator<Item = (LocalPos, T)> + ExactSizeIterator {
  #[inline]
  fn next_back(&mut self) -> Option<Self::Item> {
    map!(self.chunk, self.cells.next_back(), S)
  }

  #[inline]
  fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
    map!(self.chunk, self.cells.nth_back(n), S)
  }

  #[inline]
  fn rfold<A, F>(self, init: A, mut f: F) -> A
  where F: FnMut(A, Self::Item) -> A {
    self.cells.rfold(init, move |acc, (local, value)| {
      f(acc, (super::compose::<S>(self.chunk, local), value))
    })
  }
}

impl<T, I, const S: usize> FusedIterator for Compose<I, S>
where I: Iterator<Item = (LocalPos, T)> + FusedIterator {}
