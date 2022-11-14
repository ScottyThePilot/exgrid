use crate::chunk::*;

use std::collections::hash_map::{
  Entry, HashMap, RandomState,
  Iter as HashMapIter,
  IterMut as HashMapIterMut,
  IntoIter as HashMapIntoIter,
  Values as HashMapValues,
  ValuesMut as HashMapValuesMut,
  IntoValues as HashMapIntoValues
};
use std::hash::BuildHasher;
use std::iter::{FlatMap, FusedIterator};
use std::mem::replace;



#[derive(Debug, Clone)]
pub struct ExGridSparse<T, const S: usize, H = RandomState> {
  chunks: HashMap<[i32; 2], ChunkSparse<T, S>, H>
}

impl<T, H: BuildHasher, const S: usize> ExGridSparse<T, S, H> {
  #[inline]
  pub fn new() -> Self where H: Default {
    Self::default()
  }

  /// Gets a reference to the value of a cell if it or the chunk it is located in exists.
  pub fn get(&self, pos: [isize; 2]) -> Option<&T> {
    let (chunk, local) = decompose::<S>(pos);
    self.get_chunk(chunk)?[local].as_ref()
  }

  /// Gets a mutable reference to the value of a cell if it or the chunk it is located in exists.
  pub fn get_mut(&mut self, pos: [isize; 2]) -> Option<&mut T> {
    let (chunk, local) = decompose::<S>(pos);
    self.get_chunk_mut(chunk)?[local].as_mut()
  }

  /// Gets a mutable reference to a cell, creating a chunk if necessary.
  pub fn get_mut_default(&mut self, pos: [isize; 2]) -> &mut Option<T> {
    let (chunk, local) = decompose::<S>(pos);
    &mut self.get_chunk_default(chunk)[local]
  }

  /// Sets the value of a given cell, creating a chunk if necessary,
  /// returning any contained value if present.
  pub fn insert(&mut self, pos: [isize; 2], value: T) -> Option<T> {
    replace(self.get_mut_default(pos), Some(value))
  }

  pub fn clear(&mut self) {
    self.chunks.clear();
  }

  pub fn clean_up(&mut self) {
    self.chunks.retain(|_, chunk| !chunk.is_all_vacant());
  }

  #[doc(hidden)]
  #[deprecated = "use `is_all_vacant` instead"]
  pub fn is_vacant(&self) -> bool {
    self.is_all_vacant()
  }

  pub fn is_all_vacant(&self) -> bool {
    self.chunks.is_empty() || self.chunks.values().all(ChunkSparse::is_all_vacant)
  }

  pub fn is_all_occupied(&self) -> bool {
    !self.chunks.is_empty() && self.chunks.values().all(ChunkSparse::is_all_occupied)
  }

  /// Returns two points `(min, max)` that bound a box containing the all chunks in this grid.
  pub fn chunks_bounds(&self) -> Option<([i32; 2], [i32; 2])> {
    chunks_bounds(&self.chunks)
  }

  /// Returns two points `(min, max)` that bound a box containing all possible cells of this grid.
  /// This is "naive" because it may overestimate.
  pub fn naive_bounds(&self) -> Option<([isize; 2], [isize; 2])> {
    self.chunks_bounds().map(map_total_bounds::<S>)
  }

  #[inline]
  pub fn iter(&self) -> ExGridSparseIter<T, S> {
    self.into_iter()
  }

  #[inline]
  pub fn iter_mut(&mut self) -> ExGridSparseIterMut<T, S> {
    self.into_iter()
  }

  pub fn cells(&self) -> ExGridSparseCells<T, S> {
    let inner = self.chunks.iter().flat_map(new_sparse_cells as _);
    ExGridSparseCells { inner }
  }

  pub fn cells_mut(&mut self) -> ExGridSparseCellsMut<T, S> {
    let inner = self.chunks.iter_mut().flat_map(new_sparse_cells_mut as _);
    ExGridSparseCellsMut { inner }
  }

  pub fn into_cells(self) -> ExGridSparseIntoCells<T, S> {
    let inner = self.chunks.into_iter().flat_map(new_sparse_into_cells as _);
    ExGridSparseIntoCells { inner }
  }

  pub fn retain<F>(&mut self, f: F)
  where F: FnMut(&[i32; 2], &mut ChunkSparse<T, S>) -> bool {
    self.chunks.retain(f);
  }

  #[inline]
  pub fn get_chunk(&self, pos: [i32; 2]) -> Option<&ChunkSparse<T, S>> {
    self.chunks.get(&pos)
  }

  #[inline]
  pub fn get_chunk_mut(&mut self, pos: [i32; 2]) -> Option<&mut ChunkSparse<T, S>> {
    self.chunks.get_mut(&pos)
  }

  #[inline]
  pub fn get_chunk_default(&mut self, pos: [i32; 2]) -> &mut ChunkSparse<T, S> {
    self.get_chunk_entry(pos).or_default()
  }

  #[inline]
  pub fn get_chunk_entry(&mut self, pos: [i32; 2]) -> Entry<[i32; 2], ChunkSparse<T, S>> {
    self.chunks.entry(pos)
  }

  #[inline]
  pub fn chunks(&self) -> HashMapIter<[i32; 2], ChunkSparse<T, S>> {
    self.chunks.iter()
  }

  #[inline]
  pub fn chunks_mut(&mut self) -> HashMapIterMut<[i32; 2], ChunkSparse<T, S>> {
    self.chunks.iter_mut()
  }
}

impl<T, H: Default, const S: usize> Default for ExGridSparse<T, S, H> {
  #[inline]
  fn default() -> Self {
    ExGridSparse { chunks: HashMap::default() }
  }
}

impl<'a, T, H, const S: usize> IntoIterator for &'a ExGridSparse<T, S, H> {
  type Item = &'a T;
  type IntoIter = ExGridSparseIter<'a, T, S>;

  fn into_iter(self) -> Self::IntoIter {
    let inner = self.chunks.values().flat_map(ChunkSparse::iter as _);
    ExGridSparseIter { inner }
  }
}

impl<'a, T, H, const S: usize> IntoIterator for &'a mut ExGridSparse<T, S, H> {
  type Item = &'a mut T;
  type IntoIter = ExGridSparseIterMut<'a, T, S>;

  fn into_iter(self) -> Self::IntoIter {
    let inner = self.chunks.values_mut().flat_map(ChunkSparse::iter_mut as _);
    ExGridSparseIterMut { inner }
  }
}

impl<T, H, const S: usize> IntoIterator for ExGridSparse<T, S, H> {
  type Item = T;
  type IntoIter = ExGridSparseIntoIter<T, S>;

  fn into_iter(self) -> Self::IntoIter {
    let inner = self.chunks.into_values().flat_map(ChunkSparse::into_iter as _);
    ExGridSparseIntoIter { inner }
  }
}



#[derive(Debug, Clone)]
pub struct ExGrid<T, const S: usize, H = RandomState> {
  chunks: HashMap<[i32; 2], Chunk<T, S>, H>
}

impl<T, H: BuildHasher, const S: usize> ExGrid<T, S, H> {
  #[inline]
  pub fn new() -> Self where H: Default {
    Self::default()
  }

  /// Gets a reference to the value of a cell if the chunk it is located in exists.
  pub fn get(&self, pos: [isize; 2]) -> Option<&T> {
    let (chunk, local) = decompose::<S>(pos);
    self.chunks.get(&chunk).map(|c| &c[local])
  }

  /// Gets a mutable reference to the value of a cell if the chunk it is located in exists.
  pub fn get_mut(&mut self, pos: [isize; 2]) -> Option<&mut T> {
    let (chunk, local) = decompose::<S>(pos);
    self.chunks.get_mut(&chunk).map(|c| &mut c[local])
  }

  /// Gets a mutable reference to the value of a cell, creating a chunk if necessary.
  pub fn get_mut_default(&mut self, pos: [isize; 2]) -> &mut T
  where T: Default {
    let (chunk, local) = decompose::<S>(pos);
    &mut self.get_chunk_default(chunk)[local]
  }

  /// Sets the value of a given cell, creating a chunk if necessary,
  /// returning the previously contained value.
  pub fn insert_default(&mut self, pos: [isize; 2], value: T) -> T
  where T: Default {
    replace(self.get_mut_default(pos), value)
  }

  pub fn clear(&mut self) {
    self.chunks.clear();
  }

  /// Returns two points `(min, max)` that bound a box containing the all chunks in this grid.
  pub fn chunks_bounds(&self) -> Option<([i32; 2], [i32; 2])> {
    chunks_bounds(&self.chunks)
  }

  /// Returns two points `(min, max)` that bound a box containing all possible cells of this grid.
  pub fn bounds(&self) -> Option<([isize; 2], [isize; 2])> {
    self.chunks_bounds().map(map_total_bounds::<S>)
  }

  #[inline]
  pub fn iter(&self) -> ExGridIter<T, S> {
    self.into_iter()
  }

  #[inline]
  pub fn iter_mut(&mut self) -> ExGridIterMut<T, S> {
    self.into_iter()
  }

  pub fn cells(&self) -> ExGridCells<T, S> {
    let inner = self.chunks.iter().flat_map(new_cells as _);
    ExGridCells { inner }
  }

  pub fn cells_mut(&mut self) -> ExGridCellsMut<T, S> {
    let inner = self.chunks.iter_mut().flat_map(new_cells_mut as _);
    ExGridCellsMut { inner }
  }

  pub fn into_cells(self) -> ExGridIntoCells<T, S> {
    let inner = self.chunks.into_iter().flat_map(new_into_cells as _);
    ExGridIntoCells { inner }
  }

  pub fn retain<F>(&mut self, f: F)
  where F: FnMut(&[i32; 2], &mut Chunk<T, S>) -> bool {
    self.chunks.retain(f);
  }

  #[inline]
  pub fn get_chunk(&self, pos: [i32; 2]) -> Option<&Chunk<T, S>> {
    self.chunks.get(&pos)
  }

  #[inline]
  pub fn get_chunk_mut(&mut self, pos: [i32; 2]) -> Option<&mut Chunk<T, S>> {
    self.chunks.get_mut(&pos)
  }

  #[inline]
  pub fn get_chunk_default(&mut self, pos: [i32; 2]) -> &mut Chunk<T, S>
  where T: Default {
    self.get_chunk_entry(pos).or_default()
  }

  #[inline]
  pub fn get_chunk_entry(&mut self, pos: [i32; 2]) -> Entry<[i32; 2], Chunk<T, S>> {
    self.chunks.entry(pos)
  }

  #[inline]
  pub fn chunks(&self) -> HashMapIter<[i32; 2], Chunk<T, S>> {
    self.chunks.iter()
  }

  #[inline]
  pub fn chunks_mut(&mut self) -> HashMapIterMut<[i32; 2], Chunk<T, S>> {
    self.chunks.iter_mut()
  }
}

impl<T, H: Default, const S: usize> Default for ExGrid<T, S, H> {
  #[inline]
  fn default() -> Self {
    ExGrid { chunks: HashMap::default() }
  }
}

impl<'a, T, H, const S: usize> IntoIterator for &'a ExGrid<T, S, H> {
  type Item = &'a T;
  type IntoIter = ExGridIter<'a, T, S>;

  fn into_iter(self) -> Self::IntoIter {
    let inner = self.chunks.values().flat_map(Chunk::iter as _);
    ExGridIter { inner }
  }
}

impl<'a, T, H, const S: usize> IntoIterator for &'a mut ExGrid<T, S, H> {
  type Item = &'a mut T;
  type IntoIter = ExGridIterMut<'a, T, S>;

  fn into_iter(self) -> Self::IntoIter {
    let inner = self.chunks.values_mut().flat_map(Chunk::iter_mut as _);
    ExGridIterMut { inner }
  }
}

impl<T, H, const S: usize> IntoIterator for ExGrid<T, S, H> {
  type Item = T;
  type IntoIter = ExGridIntoIter<T, S>;

  fn into_iter(self) -> Self::IntoIter {
    let inner = self.chunks.into_values().flat_map(Chunk::into_iter as _);
    ExGridIntoIter { inner }
  }
}

/// Converts global coordinates to coordinates for a single chunk
/// and coordinates to a cell in that chunk.
pub fn decompose<const S: usize>(pos: [isize; 2]) -> ([i32; 2], [usize; 2]) {
  assert!(S > 0, "cannot index into a grid or chunk of size 0");
  let chunk = pos.map(|p| p.div_euclid(S as isize) as i32);
  let local = pos.map(|p| p.rem_euclid(S as isize) as usize);
  (chunk, local)
}

pub fn compose<const S: usize>(chunk: [i32; 2], local: [usize; 2]) -> [isize; 2] {
  assert!(S > 0, "cannot index into a grid or chunk of size 0");
  let x = chunk[0] as isize * S as isize + local[0] as isize;
  let y = chunk[1] as isize * S as isize + local[1] as isize;
  [x, y]
}

fn chunks_bounds<C, H>(chunks: &HashMap<[i32; 2], C, H>) -> Option<([i32; 2], [i32; 2])> {
  chunks.keys().fold(None, |state, &chunk| match state {
    Some((min, max)) => Some((min_pos(min, chunk), max_pos(max, chunk))),
    None => Some((chunk, chunk))
  })
}

fn map_total_bounds<const S: usize>((min, max): ([i32; 2], [i32; 2])) -> ([isize; 2], [isize; 2]) {
  (compose::<S>(min, [0, 0]), compose::<S>(max, [S - 1, S - 1]))
}

fn min_pos<T: Ord + Copy>(a: [T; 2], b: [T; 2]) -> [T; 2] {
  [std::cmp::min(a[0], b[0]), std::cmp::min(a[1], b[1])]
}

fn max_pos<T: Ord + Copy>(a: [T; 2], b: [T; 2]) -> [T; 2] {
  [std::cmp::max(a[0], b[0]), std::cmp::max(a[1], b[1])]
}



#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ExGridSparseIter<'a, T, const S: usize> {
  inner: FlatMap<
    HashMapValues<'a, [i32; 2], ChunkSparse<T, S>>,
    ChunkSparseIter<'a, T, S>,
    for<'r> fn(&'r ChunkSparse<T, S>) -> ChunkSparseIter<'r, T, S>
  >
}

impl_iterator_no_double_ended!(ExGridSparseIter, <'a, T, S>, &'a T);

#[repr(transparent)]
#[derive(Debug)]
pub struct ExGridSparseIterMut<'a, T, const S: usize> {
  inner: FlatMap<
    HashMapValuesMut<'a, [i32; 2], ChunkSparse<T, S>>,
    ChunkSparseIterMut<'a, T, S>,
    for<'r> fn(&'r mut ChunkSparse<T, S>) -> ChunkSparseIterMut<'r, T, S>
  >
}

impl_iterator_no_double_ended!(ExGridSparseIterMut, <'a, T, S>, &'a mut T);

#[repr(transparent)]
#[derive(Debug)]
pub struct ExGridSparseIntoIter<T, const S: usize> {
  inner: FlatMap<
    HashMapIntoValues<[i32; 2], ChunkSparse<T, S>>,
    ChunkSparseIntoIter<T, S>,
    fn(ChunkSparse<T, S>) -> ChunkSparseIntoIter<T, S>
  >
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

impl_iterator_no_double_ended!(ExGridSparseIntoCells, <T, S>, ([isize; 2], T));



#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct ExGridIter<'a, T, const S: usize> {
  inner: FlatMap<
    HashMapValues<'a, [i32; 2], Chunk<T, S>>,
    ChunkIter<'a, T, S>,
    for<'r> fn(&'r Chunk<T, S>) -> ChunkIter<'r, T, S>
  >
}

impl_iterator_no_double_ended!(ExGridIter, <'a, T, S>, &'a T);

#[repr(transparent)]
#[derive(Debug)]
pub struct ExGridIterMut<'a, T, const S: usize> {
  inner: FlatMap<
    HashMapValuesMut<'a, [i32; 2], Chunk<T, S>>,
    ChunkIterMut<'a, T, S>,
    for<'r> fn(&'r mut Chunk<T, S>) -> ChunkIterMut<'r, T, S>
  >
}

impl_iterator_no_double_ended!(ExGridIterMut, <'a, T, S>, &'a mut T);

#[repr(transparent)]
#[derive(Debug)]
pub struct ExGridIntoIter<T, const S: usize> {
  inner: FlatMap<
    HashMapIntoValues<[i32; 2], Chunk<T, S>>,
    ChunkIntoIter<T, S>,
    fn(Chunk<T, S>) -> ChunkIntoIter<T, S>
  >
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

impl_iterator_no_double_ended!(ExGridIntoCells, <T, S>, ([isize; 2], T));



macro_rules! map {
  ($chunk:expr, $expr:expr, $S:expr) => {
    match $expr {
      Some((local, value)) => {
        Some((compose::<$S>($chunk, local), value))
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
      f(acc, (compose::<S>(self.chunk, local), value))
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
      f(acc, (compose::<S>(self.chunk, local), value))
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
