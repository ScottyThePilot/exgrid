mod iter;

pub use self::iter::*;
use crate::chunk::*;

#[cfg(feature = "multi-thread")]
use rayon::collections::hash_map::{
  Iter as HashMapIterPar,
  IterMut as HashMapIterMutPar
};
#[cfg(feature = "multi-thread")]
use rayon::iter::{
  IntoParallelRefIterator,
  IntoParallelRefMutIterator
};

use std::collections::hash_map::{
  Entry, HashMap, RandomState,
  Iter as HashMapIter,
  IterMut as HashMapIterMut
};
use std::hash::BuildHasher;
use std::mem::replace;



#[derive(Debug, Clone)]
pub struct ExGridSparse<T, const S: usize, H = RandomState> {
  chunks: HashMap<[i32; 2], ChunkSparse<T, S>, H>
}

impl<T, H, const S: usize> ExGridSparse<T, S, H> {
  #[inline]
  pub fn new() -> Self where H: Default {
    Self::default()
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

  pub fn retain<F>(&mut self, f: F)
  where F: FnMut(&[i32; 2], &mut ChunkSparse<T, S>) -> bool {
    self.chunks.retain(f);
  }

  #[inline]
  pub fn iter(&self) -> ExGridSparseIter<T, S> {
    self.into_iter()
  }

  #[inline]
  pub fn iter_mut(&mut self) -> ExGridSparseIterMut<T, S> {
    self.into_iter()
  }

  #[inline]
  pub fn cells(&self) -> ExGridSparseCells<T, S> {
    ExGridSparseCells::new(self)
  }

  #[inline]
  pub fn cells_mut(&mut self) -> ExGridSparseCellsMut<T, S> {
    ExGridSparseCellsMut::new(self)
  }

  #[inline]
  pub fn into_cells(self) -> ExGridSparseIntoCells<T, S> {
    ExGridSparseIntoCells::new(self)
  }

  #[inline]
  pub fn chunks(&self) -> HashMapIter<[i32; 2], ChunkSparse<T, S>> {
    self.chunks.iter()
  }

  #[inline]
  pub fn chunks_mut(&mut self) -> HashMapIterMut<[i32; 2], ChunkSparse<T, S>> {
    self.chunks.iter_mut()
  }

  const NEW_SPARSE_CELLS: FilterSparseCells<T, S> = |(&chunk, i)| Compose::new(chunk, ChunkSparseCells::new(i));
  const NEW_SPARSE_CELLS_MUT: FilterSparseCellsMut<T, S> = |(&chunk, i)| Compose::new(chunk, ChunkSparseCellsMut::new(i));
  const NEW_SPARSE_INTO_CELLS: FilterSparseIntoCells<T, S> = |(chunk, i)| Compose::new(chunk, ChunkSparseIntoCells::new(i));
}

impl<T, H: BuildHasher, const S: usize> ExGridSparse<T, S, H> {
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

  pub fn get_chunk(&self, pos: [i32; 2]) -> Option<&ChunkSparse<T, S>> {
    self.chunks.get(&pos)
  }

  pub fn get_chunk_mut(&mut self, pos: [i32; 2]) -> Option<&mut ChunkSparse<T, S>> {
    self.chunks.get_mut(&pos)
  }

  pub fn get_chunk_default(&mut self, pos: [i32; 2]) -> &mut ChunkSparse<T, S> {
    self.get_chunk_entry(pos).or_default()
  }

  pub fn get_chunk_entry(&mut self, pos: [i32; 2]) -> Entry<[i32; 2], ChunkSparse<T, S>> {
    self.chunks.entry(pos)
  }

  #[cfg(feature = "multi-thread")]
  #[inline]
  pub fn par_chunks(&self) -> HashMapIterPar<[i32; 2], ChunkSparse<T, S>>
  where T: Sync {
    self.chunks.par_iter()
  }

  #[cfg(feature = "multi-thread")]
  #[inline]
  pub fn par_chunks_mut(&mut self) -> HashMapIterMutPar<[i32; 2], ChunkSparse<T, S>>
  where T: Send {
    self.chunks.par_iter_mut()
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

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    ExGridSparseIter::new(self)
  }
}

impl<'a, T, H, const S: usize> IntoIterator for &'a mut ExGridSparse<T, S, H> {
  type Item = &'a mut T;
  type IntoIter = ExGridSparseIterMut<'a, T, S>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    ExGridSparseIterMut::new(self)
  }
}

impl<T, H, const S: usize> IntoIterator for ExGridSparse<T, S, H> {
  type Item = T;
  type IntoIter = ExGridSparseIntoIter<T, S>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    ExGridSparseIntoIter::new(self)
  }
}



#[derive(Debug, Clone)]
pub struct ExGrid<T, const S: usize, H = RandomState> {
  chunks: HashMap<[i32; 2], Chunk<T, S>, H>
}

impl<T, H, const S: usize> ExGrid<T, S, H> {
  #[inline]
  pub fn new() -> Self where H: Default {
    Self::default()
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

  pub fn retain<F>(&mut self, f: F)
  where F: FnMut(&[i32; 2], &mut Chunk<T, S>) -> bool {
    self.chunks.retain(f);
  }

  #[inline]
  pub fn iter(&self) -> ExGridIter<T, S> {
    self.into_iter()
  }

  #[inline]
  pub fn iter_mut(&mut self) -> ExGridIterMut<T, S> {
    self.into_iter()
  }

  #[inline]
  pub fn cells(&self) -> ExGridCells<T, S> {
    ExGridCells::new(self)
  }

  #[inline]
  pub fn cells_mut(&mut self) -> ExGridCellsMut<T, S> {
    ExGridCellsMut::new(self)
  }

  #[inline]
  pub fn into_cells(self) -> ExGridIntoCells<T, S> {
    ExGridIntoCells::new(self)
  }

  #[inline]
  pub fn chunks(&self) -> HashMapIter<[i32; 2], Chunk<T, S>> {
    self.chunks.iter()
  }

  #[inline]
  pub fn chunks_mut(&mut self) -> HashMapIterMut<[i32; 2], Chunk<T, S>> {
    self.chunks.iter_mut()
  }

  const NEW_CELLS: FilterCells<T, S> = |(&chunk, i)| Compose::new(chunk, ChunkCells::new(i));
  const NEW_CELLS_MUT: FilterCellsMut<T, S> = |(&chunk, i)| Compose::new(chunk, ChunkCellsMut::new(i));
  const NEW_INTO_CELLS: FilterIntoCells<T, S> = |(chunk, i)| Compose::new(chunk, ChunkIntoCells::new(i));
}

impl<T, H: BuildHasher, const S: usize> ExGrid<T, S, H> {
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

  pub fn get_chunk(&self, pos: [i32; 2]) -> Option<&Chunk<T, S>> {
    self.chunks.get(&pos)
  }

  pub fn get_chunk_mut(&mut self, pos: [i32; 2]) -> Option<&mut Chunk<T, S>> {
    self.chunks.get_mut(&pos)
  }

  pub fn get_chunk_default(&mut self, pos: [i32; 2]) -> &mut Chunk<T, S>
  where T: Default {
    self.get_chunk_entry(pos).or_default()
  }

  pub fn get_chunk_entry(&mut self, pos: [i32; 2]) -> Entry<[i32; 2], Chunk<T, S>> {
    self.chunks.entry(pos)
  }

  #[cfg(feature = "multi-thread")]
  #[inline]
  pub fn par_chunks(&self) -> HashMapIterPar<[i32; 2], Chunk<T, S>>
  where T: Sync {
    self.chunks.par_iter()
  }

  #[cfg(feature = "multi-thread")]
  #[inline]
  pub fn par_chunks_mut(&mut self) -> HashMapIterMutPar<[i32; 2], Chunk<T, S>>
  where T: Send {
    self.chunks.par_iter_mut()
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

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    ExGridIter::new(self)
  }
}

impl<'a, T, H, const S: usize> IntoIterator for &'a mut ExGrid<T, S, H> {
  type Item = &'a mut T;
  type IntoIter = ExGridIterMut<'a, T, S>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    ExGridIterMut::new(self)
  }
}

impl<T, H, const S: usize> IntoIterator for ExGrid<T, S, H> {
  type Item = T;
  type IntoIter = ExGridIntoIter<T, S>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    ExGridIntoIter::new(self)
  }
}

type FilterSparseCells<T, const S: usize> = for<'r> fn((&'r [i32; 2], &'r ChunkSparse<T, S>)) -> Compose<ChunkSparseCells<'r, T, S>, S>;
type FilterSparseCellsMut<T, const S: usize> = for<'r> fn((&'r [i32; 2], &'r mut ChunkSparse<T, S>)) -> Compose<ChunkSparseCellsMut<'r, T, S>, S>;
type FilterSparseIntoCells<T, const S: usize> = fn(([i32; 2], ChunkSparse<T, S>)) -> Compose<ChunkSparseIntoCells<T, S>, S>;
type FilterCells<T, const S: usize> = for<'r> fn((&'r [i32; 2], &'r Chunk<T, S>)) -> Compose<ChunkCells<'r, T, S>, S>;
type FilterCellsMut<T, const S: usize> = for<'r> fn((&'r [i32; 2], &'r mut Chunk<T, S>)) -> Compose<ChunkCellsMut<'r, T, S>, S>;
type FilterIntoCells<T, const S: usize> = fn(([i32; 2], Chunk<T, S>)) -> Compose<ChunkIntoCells<T, S>, S>;

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
