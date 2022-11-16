use super::{ExGrid, ExGridSparse};
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
  inner: Flatten<HashMapValues<'a, [i32; 2], ChunkSparse<T, S>>>
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
  inner: Flatten<HashMapValuesMut<'a, [i32; 2], ChunkSparse<T, S>>>
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
  inner: Flatten<HashMapIntoValues<[i32; 2], ChunkSparse<T, S>>>
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
    HashMapIter<'a, [i32; 2], ChunkSparse<T, S>>,
    ComposeExtra<ChunkSparseCells<'a, T, S>, S>,
    for<'r> fn((&'r [i32; 2], &'r ChunkSparse<T, S>)) -> ComposeExtra<ChunkSparseCells<'r, T, S>, S>
  >
}

impl<'a, T, const S: usize> ExGridSparseCells<'a, T, S> {
  pub(crate) fn new<H>(grid: &'a ExGridSparse<T, S, H>) -> Self {
    let inner = grid.chunks.iter().flat_map(new_sparse_cells as _);
    ExGridSparseCells { inner }
  }
}

impl_iterator_no_double_ended!(ExGridSparseCells, <'a, T, S>, ([isize; 2], &'a T));

#[repr(transparent)]
#[derive(Debug)]
pub struct ExGridSparseCellsMut<'a, T, const S: usize> {
  inner: FlatMap<
    HashMapIterMut<'a, [i32; 2], ChunkSparse<T, S>>,
    ComposeExtra<ChunkSparseCellsMut<'a, T, S>, S>,
    for<'r> fn((&'r [i32; 2], &'r mut ChunkSparse<T, S>)) -> ComposeExtra<ChunkSparseCellsMut<'r, T, S>, S>
  >
}

impl<'a, T, const S: usize> ExGridSparseCellsMut<'a, T, S> {
  pub(crate) fn new<H>(grid: &'a mut ExGridSparse<T, S, H>) -> Self {
    let inner = grid.chunks.iter_mut().flat_map(new_sparse_cells_mut as _);
    ExGridSparseCellsMut { inner }
  }
}

impl_iterator_no_double_ended!(ExGridSparseCellsMut, <'a, T, S>, ([isize; 2], &'a mut T));

#[repr(transparent)]
#[derive(Debug)]
pub struct ExGridSparseIntoCells<T, const S: usize> {
  inner: FlatMap<
    HashMapIntoIter<[i32; 2], ChunkSparse<T, S>>,
    ComposeExtra<ChunkSparseIntoCells<T, S>, S>,
    fn(([i32; 2], ChunkSparse<T, S>)) -> ComposeExtra<ChunkSparseIntoCells<T, S>, S>
  >
}

impl<T, const S: usize> ExGridSparseIntoCells<T, S> {
  pub(crate) fn new<H>(grid: ExGridSparse<T, S, H>) -> Self {
    let inner = grid.chunks.into_iter().flat_map(new_sparse_into_cells as _);
    ExGridSparseIntoCells { inner }
  }
}

impl_iterator_no_double_ended!(ExGridSparseIntoCells, <T, S>, ([isize; 2], T));



#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ExGridIter<'a, T, const S: usize> {
  inner: Flatten<HashMapValues<'a, [i32; 2], Chunk<T, S>>>
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
  inner: Flatten<HashMapValuesMut<'a, [i32; 2], Chunk<T, S>>>
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
  inner: Flatten<HashMapIntoValues<[i32; 2], Chunk<T, S>>>
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
    HashMapIter<'a, [i32; 2], Chunk<T, S>>,
    ComposeExtra<ChunkCells<'a, T, S>, S>,
    for<'r> fn((&'r [i32; 2], &'r Chunk<T, S>)) -> ComposeExtra<ChunkCells<'r, T, S>, S>
  >
}

impl<'a, T, const S: usize> ExGridCells<'a, T, S> {
  pub(crate) fn new<H>(grid: &'a ExGrid<T, S, H>) -> Self {
    let inner = grid.chunks.iter().flat_map(new_cells as _);
    ExGridCells { inner }
  }
}

impl_iterator_no_double_ended!(ExGridCells, <'a, T, S>, ([isize; 2], &'a T));

#[repr(transparent)]
#[derive(Debug)]
pub struct ExGridCellsMut<'a, T, const S: usize> {
  inner: FlatMap<
    HashMapIterMut<'a, [i32; 2], Chunk<T, S>>,
    ComposeExtra<ChunkCellsMut<'a, T, S>, S>,
    for<'r> fn((&'r [i32; 2], &'r mut Chunk<T, S>)) -> ComposeExtra<ChunkCellsMut<'r, T, S>, S>
  >
}

impl<'a, T, const S: usize> ExGridCellsMut<'a, T, S> {
  pub(crate) fn new<H>(grid: &'a mut ExGrid<T, S, H>) -> Self {
    let inner = grid.chunks.iter_mut().flat_map(new_cells_mut as _);
    ExGridCellsMut { inner }
  }
}

impl_iterator_no_double_ended!(ExGridCellsMut, <'a, T, S>, ([isize; 2], &'a mut T));

#[repr(transparent)]
#[derive(Debug)]
pub struct ExGridIntoCells<T, const S: usize> {
  inner: FlatMap<
    HashMapIntoIter<[i32; 2], Chunk<T, S>>,
    ComposeExtra<ChunkIntoCells<T, S>, S>,
    fn(([i32; 2], Chunk<T, S>)) -> ComposeExtra<ChunkIntoCells<T, S>, S>
  >
}

impl<T, const S: usize> ExGridIntoCells<T, S> {
  pub(crate) fn new<H>(grid: ExGrid<T, S, H>) -> Self {
    let inner = grid.chunks.into_iter().flat_map(new_into_cells as _);
    ExGridIntoCells { inner }
  }
}

impl_iterator_no_double_ended!(ExGridIntoCells, <T, S>, ([isize; 2], T));



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
struct ComposeExtra<I, const S: usize> {
  chunk: [i32; 2],
  cells: I
}

impl<I, const S: usize> ComposeExtra<I, S> {
  pub fn new<T>(chunk: [i32; 2], cells: I) -> Self
  where I: Iterator<Item = ([usize; 2], T)> {
    ComposeExtra { chunk, cells }
  }
}

impl<T, I, const S: usize> Iterator for ComposeExtra<I, S>
where I: Iterator<Item = ([usize; 2], T)> {
  type Item = ([isize; 2], T);

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

impl<T, I, const S: usize> DoubleEndedIterator for ComposeExtra<I, S>
where I: DoubleEndedIterator<Item = ([usize; 2], T)> + ExactSizeIterator {
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

impl<T, I, const S: usize> FusedIterator for ComposeExtra<I, S>
where I: Iterator<Item = ([usize; 2], T)> + FusedIterator {}



#[inline]
fn new_sparse_cells<'a, T, const S: usize>(
  (&chunk, i): (&'a [i32; 2], &'a ChunkSparse<T, S>)
) -> ComposeExtra<ChunkSparseCells<'a, T, S>, S> {
  ComposeExtra::new(chunk, i.cells())
}

#[inline]
fn new_sparse_cells_mut<'a, T, const S: usize>(
  (&chunk, i): (&'a [i32; 2], &'a mut ChunkSparse<T, S>)
) -> ComposeExtra<ChunkSparseCellsMut<'a, T, S>, S> {
  ComposeExtra::new(chunk, i.cells_mut())
}

#[inline]
fn new_sparse_into_cells<T, const S: usize>(
  (chunk, i): ([i32; 2], ChunkSparse<T, S>)
) -> ComposeExtra<ChunkSparseIntoCells<T, S>, S> {
  ComposeExtra::new(chunk, i.into_cells())
}

#[inline]
fn new_cells<'a, T, const S: usize>(
  (&chunk, i): (&'a [i32; 2], &'a Chunk<T, S>)
) -> ComposeExtra<ChunkCells<'a, T, S>, S> {
  ComposeExtra::new(chunk, i.cells())
}

#[inline]
fn new_cells_mut<'a, T, const S: usize>(
  (&chunk, i): (&'a [i32; 2], &'a mut Chunk<T, S>)
) -> ComposeExtra<ChunkCellsMut<'a, T, S>, S> {
  ComposeExtra::new(chunk, i.cells_mut())
}

#[inline]
fn new_into_cells<T, const S: usize>(
  (chunk, i): ([i32; 2], Chunk<T, S>)
) -> ComposeExtra<ChunkIntoCells<T, S>, S> {
  ComposeExtra::new(chunk, i.into_cells())
}
