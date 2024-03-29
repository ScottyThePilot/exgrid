mod iter;

pub use self::iter::*;
use crate::{GlobalPos, ChunkPos, LocalPos};
use crate::chunk::*;
use crate::vector::{Lerp, Vector2};

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
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::collections::hash_map::{
  Entry, HashMap, RandomState,
  Iter as HashMapIter,
  IterMut as HashMapIterMut
};
use std::hash::BuildHasher;
use std::mem::replace;



#[derive(Debug, Clone)]
pub struct ExGridSparse<T, const S: usize, H = RandomState> {
  chunks: HashMap<ChunkPos, ChunkSparse<T, S>, H>
}

impl<T, H, const S: usize> ExGridSparse<T, S, H> {
  #[inline]
  pub fn new() -> Self where H: Default {
    Self::default()
  }

  pub fn clear(&mut self) {
    self.chunks.clear();
  }

  pub fn chunks_count(&self) -> usize {
    self.chunks.len()
  }

  pub fn cells_count_max(&self) -> usize {
    self.chunks.len() * S * S
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
  pub fn chunks_bounds(&self) -> Option<(ChunkPos, ChunkPos)> {
    chunks_bounds(&self.chunks)
  }

  /// Returns two points `(min, max)` that bound a box containing all possible cells of this grid.
  /// This is "naive" because it may overestimate.
  pub fn naive_bounds(&self) -> Option<(GlobalPos, GlobalPos)> {
    self.chunks_bounds().map(map_total_bounds::<S>)
  }

  pub fn retain<F>(&mut self, f: F)
  where F: FnMut(&ChunkPos, &mut ChunkSparse<T, S>) -> bool {
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
  pub fn chunks(&self) -> HashMapIter<ChunkPos, ChunkSparse<T, S>> {
    self.chunks.iter()
  }

  #[inline]
  pub fn chunks_mut(&mut self) -> HashMapIterMut<ChunkPos, ChunkSparse<T, S>> {
    self.chunks.iter_mut()
  }

  const NEW_SPARSE_CELLS: FilterSparseCells<T, S> = |(&chunk, i)| Compose::new(chunk, ChunkSparseCells::new(i));
  const NEW_SPARSE_CELLS_MUT: FilterSparseCellsMut<T, S> = |(&chunk, i)| Compose::new(chunk, ChunkSparseCellsMut::new(i));
  const NEW_SPARSE_INTO_CELLS: FilterSparseIntoCells<T, S> = |(chunk, i)| Compose::new(chunk, ChunkSparseIntoCells::new(i));
}

impl<T, H: BuildHasher, const S: usize> ExGridSparse<T, S, H> {
  /// Gets a reference to the value of a cell if it or the chunk it is located in exists.
  pub fn get(&self, pos: impl Into<GlobalPos>) -> Option<&T> {
    let (chunk, local) = decompose::<S>(pos.into());
    self.get_chunk(chunk)?[local].as_ref()
  }

  /// Gets a mutable reference to the value of a cell if it or the chunk it is located in exists.
  pub fn get_mut(&mut self, pos: impl Into<GlobalPos>) -> Option<&mut T> {
    let (chunk, local) = decompose::<S>(pos.into());
    self.get_chunk_mut(chunk)?[local].as_mut()
  }

  /// Gets a mutable reference to a cell, creating a chunk if necessary.
  pub fn get_mut_default(&mut self, pos: impl Into<GlobalPos>) -> &mut Option<T> {
    let (chunk, local) = decompose::<S>(pos.into());
    &mut self.get_chunk_default(chunk)[local]
  }

  pub fn get_or_insert_default(&mut self, pos: impl Into<GlobalPos>) -> &mut T where T: Default {
    self.get_mut_default(pos).get_or_insert_with(T::default)
  }

  /// Samples a value from the grid, linearly interpolating the result value.
  /// Will return `None` if any of the relevant nearby cells are empty.
  pub fn try_sample(&self, pos: impl Into<[f32; 2]>) -> Option<T>
  where T: Lerp<Output = T> + Clone {
    let pos = Vector2::from_array(pos.into());
    crate::chunk::try_sample_2d(pos, |pos: Vector2<i64>| {
      self.get(pos).cloned()
    })
  }

  /// Samples a value from the grid, linearly interpolating the result value.
  /// Uses `T`'s `Default` value whenever a relevant cell is empty.
  pub fn sample_or_default(&self, pos: impl Into<[f32; 2]>) -> T
  where T: Lerp<Output = T> + Default + Clone {
    let pos = Vector2::from_array(pos.into());
    crate::chunk::sample_2d(pos, |pos: Vector2<i64>| {
      self.get(pos).cloned().unwrap_or_default()
    })
  }

  /// Sets the value of a given cell, creating a chunk if necessary,
  /// returning any contained value if present.
  pub fn insert(&mut self, pos: impl Into<GlobalPos>, value: T) -> Option<T> {
    replace(self.get_mut_default(pos), Some(value))
  }

  pub fn contains_chunk(&self, pos: impl Into<ChunkPos>) -> bool {
    self.chunks.contains_key(&pos.into())
  }

  pub fn get_chunk(&self, pos: impl Into<ChunkPos>) -> Option<&ChunkSparse<T, S>> {
    self.chunks.get(&pos.into())
  }

  pub fn get_chunk_mut(&mut self, pos: impl Into<ChunkPos>) -> Option<&mut ChunkSparse<T, S>> {
    self.chunks.get_mut(&pos.into())
  }

  pub fn get_chunk_default(&mut self, pos: impl Into<ChunkPos>) -> &mut ChunkSparse<T, S> {
    self.get_chunk_entry(pos).or_default()
  }

  pub fn get_chunk_entry(&mut self, pos: impl Into<ChunkPos>) -> Entry<ChunkPos, ChunkSparse<T, S>> {
    self.chunks.entry(pos.into())
  }

  #[cfg(feature = "multi-thread")]
  #[inline]
  pub fn par_chunks(&self) -> HashMapIterPar<ChunkPos, ChunkSparse<T, S>>
  where T: Sync {
    self.chunks.par_iter()
  }

  #[cfg(feature = "multi-thread")]
  #[inline]
  pub fn par_chunks_mut(&mut self) -> HashMapIterMutPar<ChunkPos, ChunkSparse<T, S>>
  where T: Send {
    self.chunks.par_iter_mut()
  }

  pub fn entry(&mut self, pos: impl Into<GlobalPos>) -> ExGridSparseEntry<T, S> {
    let (chunk, local) = decompose::<S>(pos.into());
    ExGridSparseEntry {
      entry: self.chunks.entry(chunk),
      pos: local
    }
  }
}

impl<T, H: Default, const S: usize> Default for ExGridSparse<T, S, H> {
  #[inline]
  fn default() -> Self {
    ExGridSparse { chunks: HashMap::default() }
  }
}

impl<T: Eq, H: BuildHasher, const S: usize> Eq for ExGridSparse<T, S, H> {}

impl<T: PartialEq, H: BuildHasher, const S: usize> PartialEq for ExGridSparse<T, S, H> {
  fn eq(&self, other: &Self) -> bool {
    self.chunks == other.chunks
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

#[cfg(feature = "serde")]
impl<T, const L: usize, H> Serialize for ExGridSparse<T, L, H>
where T: Serialize {
  #[inline]
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    HashMap::serialize(&self.chunks, serializer)
  }
}

#[cfg(feature = "serde")]
impl<'de, T, const L: usize, H> Deserialize<'de> for ExGridSparse<T, L, H>
where T: Deserialize<'de>, H: BuildHasher + Default {
  #[inline]
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    HashMap::deserialize(deserializer).map(|chunks| ExGridSparse { chunks })
  }
}

#[derive(Debug)]
pub struct ExGridSparseEntry<'a, T, const S: usize> {
  entry: Entry<'a, ChunkPos, ChunkSparse<T, S>>,
  pos: LocalPos
}

impl<'a, T, const S: usize> ExGridSparseEntry<'a, T, S> {
  pub fn or_insert(self, default: ChunkSparse<T, S>) -> &'a mut Option<T> {
    &mut self.entry.or_insert(default)[self.pos]
  }

  pub fn or_insert_with<F: FnOnce() -> ChunkSparse<T, S>>(self, default: F) -> &'a mut Option<T> {
    &mut self.entry.or_insert_with(default)[self.pos]
  }

  pub fn or_insert_with_key<F: FnOnce(ChunkPos) -> ChunkSparse<T, S>>(self, default: F) -> &'a mut Option<T> {
    &mut self.entry.or_insert_with_key(move |&k| default(k))[self.pos]
  }
}



#[derive(Debug, Clone)]
pub struct ExGrid<T, const S: usize, H = RandomState> {
  chunks: HashMap<ChunkPos, Chunk<T, S>, H>
}

impl<T, H, const S: usize> ExGrid<T, S, H> {
  #[inline]
  pub fn new() -> Self where H: Default {
    Self::default()
  }

  pub fn clear(&mut self) {
    self.chunks.clear();
  }

  pub fn chunks_count(&self) -> usize {
    self.chunks.len()
  }

  pub fn cells_count(&self) -> usize {
    self.chunks.len() * S * S
  }

  /// Returns two points `(min, max)` that bound a box containing the all chunks in this grid.
  pub fn chunks_bounds(&self) -> Option<(ChunkPos, ChunkPos)> {
    chunks_bounds(&self.chunks)
  }

  /// Returns two points `(min, max)` that bound a box containing all possible cells of this grid.
  pub fn bounds(&self) -> Option<(GlobalPos, GlobalPos)> {
    self.chunks_bounds().map(map_total_bounds::<S>)
  }

  pub fn retain<F>(&mut self, f: F)
  where F: FnMut(&ChunkPos, &mut Chunk<T, S>) -> bool {
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
  pub fn chunks(&self) -> HashMapIter<ChunkPos, Chunk<T, S>> {
    self.chunks.iter()
  }

  #[inline]
  pub fn chunks_mut(&mut self) -> HashMapIterMut<ChunkPos, Chunk<T, S>> {
    self.chunks.iter_mut()
  }

  const NEW_CELLS: FilterCells<T, S> = |(&chunk, i)| Compose::new(chunk, ChunkCells::new(i));
  const NEW_CELLS_MUT: FilterCellsMut<T, S> = |(&chunk, i)| Compose::new(chunk, ChunkCellsMut::new(i));
  const NEW_INTO_CELLS: FilterIntoCells<T, S> = |(chunk, i)| Compose::new(chunk, ChunkIntoCells::new(i));
}

impl<T, H: BuildHasher, const S: usize> ExGrid<T, S, H> {
  /// Gets a reference to the value of a cell if the chunk it is located in exists.
  pub fn get(&self, pos: impl Into<GlobalPos>) -> Option<&T> {
    let (chunk, local) = decompose::<S>(pos.into());
    self.chunks.get(&chunk).map(|c| &c[local])
  }

  /// Gets a mutable reference to the value of a cell if the chunk it is located in exists.
  pub fn get_mut(&mut self, pos: impl Into<GlobalPos>) -> Option<&mut T> {
    let (chunk, local) = decompose::<S>(pos.into());
    self.chunks.get_mut(&chunk).map(|c| &mut c[local])
  }

  /// Gets a mutable reference to the value of a cell, creating a chunk if necessary.
  pub fn get_mut_default(&mut self, pos: impl Into<GlobalPos>) -> &mut T
  where T: Default {
    let (chunk, local) = decompose::<S>(pos.into());
    &mut self.get_chunk_default(chunk)[local]
  }

  /// Samples a value from the grid, linearly interpolating the result value.
  /// Will return `None` if any of the relevant nearby cells are empty.
  pub fn try_sample(&self, pos: impl Into<[f32; 2]>) -> Option<T>
  where T: Lerp<Output = T> + Clone {
    let pos = Vector2::from_array(pos.into());
    crate::chunk::try_sample_2d(pos, |pos: Vector2<i64>| {
      self.get(pos).cloned()
    })
  }

  /// Samples a value from the grid, linearly interpolating the result value.
  /// Uses `T`'s `Default` value whenever a relevant cell is empty.
  pub fn sample_or_default(&self, pos: impl Into<[f32; 2]>) -> T
  where T: Lerp<Output = T> + Default + Clone {
    let pos = Vector2::from_array(pos.into());
    crate::chunk::sample_2d(pos, |pos: Vector2<i64>| {
      self.get(pos).cloned().unwrap_or_default()
    })
  }

  /// Samples a value from the grid, linearly interpolating the result value.
  /// Uses `T`'s `Default` value whenever a relevant cell is empty,
  /// inserting that value into the empty cell.
  pub fn sample_insert_default(&mut self, pos: impl Into<[f32; 2]>) -> T
  where T: Lerp<Output = T> + Default + Clone {
    let pos = Vector2::from_array(pos.into());
    crate::chunk::sample_2d(pos, |pos: Vector2<i64>| {
      self.get_mut_default(pos).clone()
    })
  }

  /// Sets the value of a given cell, creating a chunk if necessary,
  /// returning the previously contained value.
  pub fn insert_default(&mut self, pos: impl Into<GlobalPos>, value: T) -> T
  where T: Default {
    replace(self.get_mut_default(pos), value)
  }

  pub fn contains_chunk(&self, pos: impl Into<ChunkPos>) -> bool {
    self.chunks.contains_key(&pos.into())
  }

  pub fn get_chunk(&self, pos: impl Into<ChunkPos>) -> Option<&Chunk<T, S>> {
    self.chunks.get(&pos.into())
  }

  pub fn get_chunk_mut(&mut self, pos: impl Into<ChunkPos>) -> Option<&mut Chunk<T, S>> {
    self.chunks.get_mut(&pos.into())
  }

  pub fn get_chunk_default(&mut self, pos: impl Into<ChunkPos>) -> &mut Chunk<T, S>
  where T: Default {
    self.get_chunk_entry(pos).or_default()
  }

  pub fn get_chunk_entry(&mut self, pos: impl Into<ChunkPos>) -> Entry<ChunkPos, Chunk<T, S>> {
    self.chunks.entry(pos.into())
  }

  #[cfg(feature = "multi-thread")]
  #[inline]
  pub fn par_chunks(&self) -> HashMapIterPar<ChunkPos, Chunk<T, S>>
  where T: Sync {
    self.chunks.par_iter()
  }

  #[cfg(feature = "multi-thread")]
  #[inline]
  pub fn par_chunks_mut(&mut self) -> HashMapIterMutPar<ChunkPos, Chunk<T, S>>
  where T: Send {
    self.chunks.par_iter_mut()
  }

  pub fn entry(&mut self, pos: impl Into<GlobalPos>) -> ExGridEntry<T, S> {
    let (chunk, local) = decompose::<S>(pos.into());
    ExGridEntry {
      entry: self.chunks.entry(chunk),
      pos: local
    }
  }
}

impl<T, H: Default, const S: usize> Default for ExGrid<T, S, H> {
  #[inline]
  fn default() -> Self {
    ExGrid { chunks: HashMap::default() }
  }
}

impl<T: Eq, H: BuildHasher, const S: usize> Eq for ExGrid<T, S, H> {}

impl<T: PartialEq, H: BuildHasher, const S: usize> PartialEq for ExGrid<T, S, H> {
  fn eq(&self, other: &Self) -> bool {
    self.chunks == other.chunks
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

#[cfg(feature = "serde")]
impl<T, const L: usize, H> Serialize for ExGrid<T, L, H>
where T: Serialize {
  #[inline]
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    HashMap::serialize(&self.chunks, serializer)
  }
}

#[cfg(feature = "serde")]
impl<'de, T, const L: usize, H> Deserialize<'de> for ExGrid<T, L, H>
where T: Deserialize<'de>, H: BuildHasher + Default {
  #[inline]
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    HashMap::deserialize(deserializer).map(|chunks| ExGrid { chunks })
  }
}

#[derive(Debug)]
pub struct ExGridEntry<'a, T, const S: usize> {
  entry: Entry<'a, ChunkPos, Chunk<T, S>>,
  pos: LocalPos
}

impl<'a, T, const S: usize> ExGridEntry<'a, T, S> {
  pub fn or_insert(self, default: Chunk<T, S>) -> &'a mut T {
    &mut self.entry.or_insert(default)[self.pos]
  }

  pub fn or_insert_with<F: FnOnce() -> Chunk<T, S>>(self, default: F) -> &'a mut T {
    &mut self.entry.or_insert_with(default)[self.pos]
  }

  pub fn or_insert_with_key<F: FnOnce(ChunkPos) -> Chunk<T, S>>(self, default: F) -> &'a mut T {
    &mut self.entry.or_insert_with_key(move |&k| default(k))[self.pos]
  }
}



type FilterSparseCells<T, const S: usize> = for<'r> fn((&'r ChunkPos, &'r ChunkSparse<T, S>)) -> Compose<ChunkSparseCells<'r, T, S>, S>;
type FilterSparseCellsMut<T, const S: usize> = for<'r> fn((&'r ChunkPos, &'r mut ChunkSparse<T, S>)) -> Compose<ChunkSparseCellsMut<'r, T, S>, S>;
type FilterSparseIntoCells<T, const S: usize> = fn((ChunkPos, ChunkSparse<T, S>)) -> Compose<ChunkSparseIntoCells<T, S>, S>;
type FilterCells<T, const S: usize> = for<'r> fn((&'r ChunkPos, &'r Chunk<T, S>)) -> Compose<ChunkCells<'r, T, S>, S>;
type FilterCellsMut<T, const S: usize> = for<'r> fn((&'r ChunkPos, &'r mut Chunk<T, S>)) -> Compose<ChunkCellsMut<'r, T, S>, S>;
type FilterIntoCells<T, const S: usize> = fn((ChunkPos, Chunk<T, S>)) -> Compose<ChunkIntoCells<T, S>, S>;

/// Converts global coordinates to coordinates for a single chunk
/// and coordinates to a cell in that chunk.
pub fn decompose<const S: usize>(pos: GlobalPos) -> (ChunkPos, LocalPos) {
  assert!(S > 0, "cannot index into a grid or chunk of size 0");
  let chunk = pos.map(|p| p.div_euclid(S as i64) as i32);
  let local = pos.map(|p| p.rem_euclid(S as i64) as usize);
  (chunk, local)
}

pub fn compose<const S: usize>(chunk: ChunkPos, local: LocalPos) -> GlobalPos {
  assert!(S > 0, "cannot index into a grid or chunk of size 0");
  let chunk = Vector2::from_array(chunk);
  let local = Vector2::from_array(local);
  Vector2::into_array(chunk.cast::<i64>() * S as i64 + local.cast::<i64>())
}

fn chunks_bounds<C, H>(chunks: &HashMap<ChunkPos, C, H>) -> Option<(ChunkPos, ChunkPos)> {
  chunks.keys().fold(None, |state, &chunk| match state {
    Some((min, max)) => Some((ChunkPos::min(min, chunk), ChunkPos::max(max, chunk))),
    None => Some((chunk, chunk))
  })
}

fn map_total_bounds<const S: usize>((min, max): (ChunkPos, ChunkPos)) -> (GlobalPos, GlobalPos) {
  (compose::<S>(min, [0; 2]), compose::<S>(max, [S - 1; 2]))
}
