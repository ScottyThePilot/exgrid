mod iter;

pub use self::iter::*;
#[cfg(feature = "serde")]
use crate::nested_array::{Array3NestedRepr, Array3Nested};
use super::LocalPos;
use crate::misc::{
  from_3nested_array,
  from_3nested_array_ref,
  from_3nested_array_mut
};
use crate::vector::Vector3;

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::ops::{Index, IndexMut};



#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkSparse<T, const S: usize> {
  inner: Chunk<Option<T>, S>
}

impl<T, const S: usize> ChunkSparse<T, S> {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn init<F: FnMut(LocalPos) -> Option<T>>(f: F) -> Self {
    ChunkSparse { inner: Chunk::init(f) }
  }

  pub fn map_dense<U, F>(self, f: F) -> Chunk<U, S>
  where F: FnMut(Option<T>) -> U {
    self.inner.map(f)
  }

  pub fn map_sparse<U, F>(self, mut f: F) -> ChunkSparse<U, S>
  where F: FnMut(T) -> U {
    self.map(|cell| cell.map(&mut f))
  }

  pub fn map<U, F>(self, f: F) -> ChunkSparse<U, S>
  where F: FnMut(Option<T>) -> Option<U> {
    ChunkSparse { inner: self.inner.map(f) }
  }

  pub fn as_chunk(&self) -> &Chunk<Option<T>, S> {
    &self.inner
  }

  pub fn as_chunk_mut(&mut self) -> &mut Chunk<Option<T>, S> {
    &mut self.inner
  }

  #[inline]
  pub fn get(&self, pos: impl Into<LocalPos>) -> &Option<T> {
    &self[pos.into()]
  }

  pub fn to_vec(&self) -> Vec<Option<T>> where T: Clone {
    self.inner.to_vec()
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

  #[inline]
  pub fn iter(&self) -> ChunkSparseIter<T, S> {
    self.into_iter()
  }

  #[inline]
  pub fn iter_mut(&mut self) -> ChunkSparseIterMut<T, S> {
    self.into_iter()
  }

  #[inline]
  pub fn cells(&self) -> ChunkSparseCells<T, S> {
    ChunkSparseCells::new(self)
  }

  #[inline]
  pub fn cells_mut(&mut self) -> ChunkSparseCellsMut<T, S> {
    ChunkSparseCellsMut::new(self)
  }

  #[inline]
  pub fn into_cells(self) -> ChunkSparseIntoCells<T, S> {
    ChunkSparseIntoCells::new(self)
  }

  // Functions for filtering iterator output
  const NEW_CELLS: FilterCells<T, S> = |(i, v)| v.as_ref().map(|v| (i, v));
  const NEW_CELLS_MUT: FilterCellsMut<T, S> = |(i, v)| v.as_mut().map(|v| (i, v));
  const NEW_INTO_CELLS: FilterIntoCells<T, S> = |(i, v)| v.map(|v| (i, v));
}

impl<T, const S: usize> Index<Vector3<usize>> for ChunkSparse<T, S> {
  type Output = Option<T>;

  #[inline]
  fn index(&self, pos: Vector3<usize>) -> &Option<T> {
    &self.inner[pos]
  }
}

impl<T, const S: usize> IndexMut<Vector3<usize>> for ChunkSparse<T, S> {
  #[inline]
  fn index_mut(&mut self, pos: Vector3<usize>) -> &mut Option<T> {
    &mut self.inner[pos]
  }
}

impl<T, const S: usize> Index<LocalPos> for ChunkSparse<T, S> {
  type Output = Option<T>;

  #[inline]
  fn index(&self, pos: LocalPos) -> &Option<T> {
    &self.inner[pos]
  }
}

impl<T, const S: usize> IndexMut<LocalPos> for ChunkSparse<T, S> {
  #[inline]
  fn index_mut(&mut self, pos: LocalPos) -> &mut Option<T> {
    &mut self.inner[pos]
  }
}

impl<T, const S: usize> Default for ChunkSparse<T, S> {
  #[inline]
  fn default() -> Self {
    ChunkSparse { inner: Chunk::default() }
  }
}

impl<T, const S: usize> From<[[[Option<T>; S]; S]; S]> for ChunkSparse<T, S> {
  fn from(inner: [[[Option<T>; S]; S]; S]) -> Self {
    ChunkSparse { inner: Chunk { inner } }
  }
}

impl<T, const S: usize> From<ChunkSparse<T, S>> for [[[Option<T>; S]; S]; S] {
  fn from(chunk: ChunkSparse<T, S>) -> Self {
    chunk.inner.inner
  }
}

impl<T, const S: usize> From<ChunkSparse<T, S>> for Box<[Option<T>]> {
  fn from(chunk: ChunkSparse<T, S>) -> Self {
    from_3nested_array(chunk.inner.inner)
  }
}

impl<T, const S: usize> From<Chunk<Option<T>, S>> for ChunkSparse<T, S> {
  fn from(inner: Chunk<Option<T>, S>) -> Self {
    ChunkSparse { inner }
  }
}

impl<T, const S: usize> From<ChunkSparse<T, S>> for Chunk<Option<T>, S> {
  fn from(chunk: ChunkSparse<T, S>) -> Self {
    chunk.inner
  }
}

impl<'a, T, const S: usize> IntoIterator for &'a ChunkSparse<T, S> {
  type Item = &'a T;
  type IntoIter = ChunkSparseIter<'a, T, S>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    ChunkSparseIter::new(self)
  }
}

impl<'a, T, const S: usize> IntoIterator for &'a mut ChunkSparse<T, S> {
  type Item = &'a mut T;
  type IntoIter = ChunkSparseIterMut<'a, T, S>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    ChunkSparseIterMut::new(self)
  }
}

impl<T, const S: usize> IntoIterator for ChunkSparse<T, S> {
  type Item = T;
  type IntoIter = ChunkSparseIntoIter<T, S>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    ChunkSparseIntoIter::new(self)
  }
}

#[cfg(feature = "serde")]
impl<T, const L: usize> Serialize for ChunkSparse<T, L>
where T: Serialize {
  #[inline]
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    Chunk::serialize(&self.inner, serializer)
  }
}

#[cfg(feature = "serde")]
impl<'de, T, const L: usize> Deserialize<'de> for ChunkSparse<T, L>
where T: Deserialize<'de> {
  #[inline]
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    Chunk::deserialize(deserializer).map(ChunkSparse::from)
  }
}

type FilterCells<T, const S: usize> = for<'a> fn((LocalPos, &'a Option<T>)) -> Option<(LocalPos, &'a T)>;
type FilterCellsMut<T, const S: usize> = for<'a> fn((LocalPos, &'a mut Option<T>)) -> Option<(LocalPos, &'a mut T)>;
type FilterIntoCells<T, const S: usize> = fn((LocalPos, Option<T>)) -> Option<(LocalPos, T)>;



#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Chunk<T, const S: usize> {
  inner: [[[T; S]; S]; S]
}

impl<T, const S: usize> Chunk<T, S> {
  pub fn new() -> Self where T: Default {
    Self::default()
  }

  pub fn init<F: FnMut(LocalPos) -> T>(f: F) -> Self {
    Chunk { inner: new_inner(f) }
  }

  pub fn map<F, U>(self, mut f: F) -> Chunk<U, S>
  where F: FnMut(T) -> U {
    Chunk { inner: self.inner.map(|slice| slice.map(|slice| slice.map(&mut f))) }
  }

  #[inline]
  pub fn get(&self, pos: impl Into<LocalPos>) -> &T {
    &self[pos.into()]
  }

  pub fn to_vec(&self) -> Vec<T> where T: Clone {
    Vec::from(from_3nested_array(self.inner.clone()))
  }

  #[inline]
  pub fn iter(&self) -> ChunkIter<T, S> {
    self.into_iter()
  }

  #[inline]
  pub fn iter_mut(&mut self) -> ChunkIterMut<T, S> {
    self.into_iter()
  }

  #[inline]
  pub fn cells(&self) -> ChunkCells<T, S> {
    ChunkCells::new(self)
  }

  #[inline]
  pub fn cells_mut(&mut self) -> ChunkCellsMut<T, S> {
    ChunkCellsMut::new(self)
  }

  #[inline]
  pub fn into_cells(self) -> ChunkIntoCells<T, S> {
    ChunkIntoCells::new(self)
  }

  fn assert_bounds_f(pos: Vector3<f32>) {
    let in_bounds = pos.x >= 0.0 && pos.y >= 0.0 && pos.x < S as f32 && pos.y < S as f32;
    assert!(in_bounds, "position out of bound: the size is {S} but the position is {}, {}", pos.x, pos.y);
  }

  fn assert_bounds_u(pos: Vector3<usize>) {
    let in_bounds = pos.x < S && pos.y < S;
    assert!(in_bounds, "position out of bound: the size is {S} but the position is {}, {}", pos.x, pos.y)
  }
}

impl<T, const S: usize> Index<Vector3<usize>> for Chunk<T, S> {
  type Output = T;

  #[inline]
  fn index(&self, pos: Vector3<usize>) -> &T {
    Self::assert_bounds_u(pos);
    &self.inner[pos.z][pos.y][pos.x]
  }
}

impl<T, const S: usize> IndexMut<Vector3<usize>> for Chunk<T, S> {
  #[inline]
  fn index_mut(&mut self, pos: Vector3<usize>) -> &mut T {
    Self::assert_bounds_u(pos);
    &mut self.inner[pos.z][pos.y][pos.x]
  }
}

impl<T, const S: usize> Index<LocalPos> for Chunk<T, S> {
  type Output = T;

  #[inline]
  fn index(&self, pos: LocalPos) -> &T {
    &self[Vector3::from_array(pos)]
  }
}

impl<T, const S: usize> IndexMut<LocalPos> for Chunk<T, S> {
  #[inline]
  fn index_mut(&mut self, pos: LocalPos) -> &mut T {
    &mut self[Vector3::from_array(pos)]
  }
}

impl<T: Default, const S: usize> Default for Chunk<T, S> {
  #[inline]
  fn default() -> Self {
    Chunk { inner: new_inner(|_| T::default()) }
  }
}

impl<T, const S: usize> From<[[[T; S]; S]; S]> for Chunk<T, S> {
  fn from(inner: [[[T; S]; S]; S]) -> Self {
    Chunk { inner }
  }
}

impl<T, const S: usize> From<Chunk<T, S>> for [[[T; S]; S]; S] {
  fn from(chunk: Chunk<T, S>) -> Self {
    chunk.inner
  }
}

impl<T, const S: usize> From<Chunk<T, S>> for Box<[T]> {
  fn from(chunk: Chunk<T, S>) -> Self {
    from_3nested_array(chunk.inner)
  }
}

impl<'a, T, const S: usize> IntoIterator for &'a Chunk<T, S> {
  type Item = &'a T;
  type IntoIter = ChunkIter<'a, T, S>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    ChunkIter::new(self)
  }
}

impl<'a, T, const S: usize> IntoIterator for &'a mut Chunk<T, S> {
  type Item = &'a mut T;
  type IntoIter = ChunkIterMut<'a, T, S>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    ChunkIterMut::new(self)
  }
}

impl<T, const S: usize> IntoIterator for Chunk<T, S> {
  type Item = T;
  type IntoIter = ChunkIntoIter<T, S>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    ChunkIntoIter::new(self)
  }
}

#[cfg(feature = "serde")]
impl<T, const L: usize> Serialize for Chunk<T, L>
where T: Serialize {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    let nested_array = Array3NestedRepr::from_array_ref(&self.inner);
    <Array3Nested<T, L>>::serialize(nested_array, serializer)
  }
}

#[cfg(feature = "serde")]
impl<'de, T, const L: usize> Deserialize<'de> for Chunk<T, L>
where T: Deserialize<'de> {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    <Array3Nested<T, L>>::deserialize(deserializer).map(|nested_array| {
      Chunk { inner: Array3NestedRepr::into_array(nested_array) }
    })
  }
}

// This is necessary due to the array primitive's `Default` impl not actually being generic across all `N`.
fn new_inner<T, F: FnMut(LocalPos) -> T, const N: usize>(mut f: F) -> [[[T; N]; N]; N] {
  std::array::from_fn(|z| {
    std::array::from_fn(|y| {
      std::array::from_fn(|x| {
        f([x, y, z])
      })
    })
  })
}
