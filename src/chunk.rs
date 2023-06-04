mod iter;
#[cfg(feature = "multi-thread")]
mod iter_par;
#[cfg(feature = "serde")]
mod nested_array;

pub use self::iter::*;
#[cfg(feature = "multi-thread")]
pub use self::iter_par::*;
#[cfg(feature = "serde")]
use self::nested_array::NestedArray;
use crate::LocalPos;
use crate::vector::{Lerp, Vector2};

use num_traits::AsPrimitive;
#[cfg(feature = "multi-thread")]
use rayon::iter::{
  IntoParallelIterator,
  IntoParallelRefIterator,
  IntoParallelRefMutIterator,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::ops::{Index, IndexMut};



#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

  pub fn sample(&self, pos: impl Into<[f32; 2]>) -> Option<T>
  where T: Lerp<Output = T> + Clone {
    let pos = Vector2::from_array(pos.into());
    Chunk::<T, S>::assert_bounds_f(pos);
    try_sample_2d(pos, |pos: Vector2<usize>| {
      self[pos].as_ref().cloned()
    })
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

  pub fn horizontal_slice(&self, y: usize) -> [Option<T>; S]
  where T: Clone {
    self.inner.horizontal_slice(y)
  }

  pub(crate) fn horizontal_slice_iter(&self, y: usize) -> impl Iterator<Item = &Option<T>> {
    self.inner.horizontal_slice_iter(y)
  }

  pub fn vertical_slice(&self, x: usize) -> [Option<T>; S]
  where T: Clone {
    self.inner.vertical_slice(x)
  }

  pub(crate) fn vertical_slice_iter(&self, x: usize) -> impl Iterator<Item = &Option<T>> {
    self.inner.vertical_slice_iter(x)
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

impl<T, const S: usize> Index<Vector2<usize>> for ChunkSparse<T, S> {
  type Output = Option<T>;

  #[inline]
  fn index(&self, pos: Vector2<usize>) -> &Option<T> {
    &self.inner[pos]
  }
}

impl<T, const S: usize> IndexMut<Vector2<usize>> for ChunkSparse<T, S> {
  #[inline]
  fn index_mut(&mut self, pos: Vector2<usize>) -> &mut Option<T> {
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

impl<T, const S: usize> From<[[Option<T>; S]; S]> for ChunkSparse<T, S> {
  fn from(inner: [[Option<T>; S]; S]) -> Self {
    ChunkSparse { inner: Chunk { inner } }
  }
}

impl<T, const S: usize> From<ChunkSparse<T, S>> for [[Option<T>; S]; S] {
  fn from(chunk: ChunkSparse<T, S>) -> Self {
    chunk.inner.inner
  }
}

impl<T, const S: usize> From<ChunkSparse<T, S>> for Box<[Option<T>]> {
  fn from(chunk: ChunkSparse<T, S>) -> Self {
    from_nested_array(chunk.inner.inner)
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

#[cfg(feature = "multi-thread")]
impl<'data, T: Sync + 'data, const S: usize> IntoParallelRefIterator<'data> for ChunkSparse<T, S> {
  type Item = &'data T;
  type Iter = ChunkSparseIterPar<'data, T, S>;

  #[inline]
  fn par_iter(&'data self) -> Self::Iter {
    ChunkSparseIterPar::new(&self)
  }
}

#[cfg(feature = "multi-thread")]
impl<'data, T: Send + 'data, const S: usize> IntoParallelRefMutIterator<'data> for ChunkSparse<T, S> {
  type Item = &'data mut T;
  type Iter = ChunkSparseIterMutPar<'data, T, S>;

  #[inline]
  fn par_iter_mut(&'data mut self) -> Self::Iter {
    ChunkSparseIterMutPar::new(self)
  }
}

#[cfg(feature = "multi-thread")]
impl<T: Send, const S: usize> IntoParallelIterator for ChunkSparse<T, S> {
  type Item = T;
  type Iter = ChunkSparseIntoIterPar<T, S>;

  #[inline]
  fn into_par_iter(self) -> Self::Iter {
    ChunkSparseIntoIterPar::new(self)
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Chunk<T, const S: usize> {
  inner: [[T; S]; S]
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
    Chunk { inner: self.inner.map(|slice| slice.map(&mut f)) }
  }

  #[inline]
  pub fn get(&self, pos: impl Into<LocalPos>) -> &T {
    &self[pos.into()]
  }

  pub fn sample(&self, pos: impl Into<[f32; 2]>) -> T
  where T: Lerp<Output = T> + Clone {
    let pos = Vector2::from_array(pos.into());
    Self::assert_bounds_f(pos);
    sample_2d(pos, |pos: Vector2<usize>| {
      self[pos].clone()
    })
  }

  pub fn to_vec(&self) -> Vec<T> where T: Clone {
    Vec::from(from_nested_array(self.inner.clone()))
  }

  pub fn horizontal_slice(&self, y: usize) -> [T; S] where T: Clone {
    Self::assert_bounds_horizontal(y);
    self.inner[y].clone()
  }

  pub fn horizontal_slice_ref(&self, y: usize) -> &[T; S] {
    Self::assert_bounds_horizontal(y);
    &self.inner[y]
  }

  pub(crate) fn horizontal_slice_iter(&self, y: usize) -> impl Iterator<Item = &T> {
    Self::assert_bounds_horizontal(y);
    self.inner[y].iter()
  }

  pub fn vertical_slice(&self, x: usize) -> [T; S] where T: Clone {
    Self::assert_bounds_vertical(x);
    std::array::from_fn(|y| self.inner[y][x].clone())
  }

  pub fn vertical_slice_each_ref(&self, x: usize) -> [&T; S] {
    Self::assert_bounds_vertical(x);
    std::array::from_fn(|y| &self.inner[y][x])
  }

  pub(crate) fn vertical_slice_iter(&self, x: usize) -> impl Iterator<Item = &T> {
    Self::assert_bounds_vertical(x);
    (0..S).map(move |y| &self.inner[y][x])
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

  fn assert_bounds_f(pos: Vector2<f32>) {
    let in_bounds = pos.x >= 0.0 && pos.y >= 0.0 && pos.x < S as f32 && pos.y < S as f32;
    assert!(in_bounds, "position out of bound: the size is {S} but the position is {}, {}", pos.x, pos.y);
  }

  fn assert_bounds_u(pos: Vector2<usize>) {
    let in_bounds = pos.x < S && pos.y < S;
    assert!(in_bounds, "position out of bound: the size is {S} but the position is {}, {}", pos.x, pos.y)
  }

  fn assert_bounds_horizontal(y: usize) {
    assert!(y < S, "position out of bounds: the size is {S} but the y-index is {y}");
  }

  fn assert_bounds_vertical(x: usize) {
    assert!(x < S, "position out of bounds: the size is {S} but the x-index is {x}");
  }
}

impl<T, const S: usize> Index<Vector2<usize>> for Chunk<T, S> {
  type Output = T;

  #[inline]
  fn index(&self, pos: Vector2<usize>) -> &T {
    Self::assert_bounds_u(pos);
    &self.inner[pos.y][pos.x]
  }
}

impl<T, const S: usize> IndexMut<Vector2<usize>> for Chunk<T, S> {
  #[inline]
  fn index_mut(&mut self, pos: Vector2<usize>) -> &mut T {
    Self::assert_bounds_u(pos);
    &mut self.inner[pos.y][pos.x]
  }
}

impl<T, const S: usize> Index<LocalPos> for Chunk<T, S> {
  type Output = T;

  #[inline]
  fn index(&self, pos: LocalPos) -> &T {
    &self[Vector2::from_array(pos)]
  }
}

impl<T, const S: usize> IndexMut<LocalPos> for Chunk<T, S> {
  #[inline]
  fn index_mut(&mut self, pos: LocalPos) -> &mut T {
    &mut self[Vector2::from_array(pos)]
  }
}

impl<T: Default, const S: usize> Default for Chunk<T, S> {
  #[inline]
  fn default() -> Self {
    Chunk { inner: new_inner(|_| T::default()) }
  }
}

impl<T, const S: usize> From<[[T; S]; S]> for Chunk<T, S> {
  fn from(inner: [[T; S]; S]) -> Self {
    Chunk { inner }
  }
}

impl<T, const S: usize> From<Chunk<T, S>> for [[T; S]; S] {
  fn from(chunk: Chunk<T, S>) -> Self {
    chunk.inner
  }
}

impl<T, const S: usize> From<Chunk<T, S>> for Box<[T]> {
  fn from(chunk: Chunk<T, S>) -> Self {
    from_nested_array(chunk.inner)
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

#[cfg(feature = "multi-thread")]
impl<'data, T: Sync + 'data, const S: usize> IntoParallelRefIterator<'data> for Chunk<T, S> {
  type Item = &'data T;
  type Iter = ChunkIterPar<'data, T, S>;

  #[inline]
  fn par_iter(&'data self) -> Self::Iter {
    ChunkIterPar::new(&self)
  }
}

#[cfg(feature = "multi-thread")]
impl<'data, T: Send + 'data, const S: usize> IntoParallelRefMutIterator<'data> for Chunk<T, S> {
  type Item = &'data mut T;
  type Iter = ChunkIterMutPar<'data, T, S>;

  #[inline]
  fn par_iter_mut(&'data mut self) -> Self::Iter {
    ChunkIterMutPar::new(self)
  }
}

#[cfg(feature = "multi-thread")]
impl<T: Send, const S: usize> IntoParallelIterator for Chunk<T, S> {
  type Item = T;
  type Iter = ChunkIntoIterPar<T, S>;

  #[inline]
  fn into_par_iter(self) -> Self::Iter {
    ChunkIntoIterPar::new(self)
  }
}

#[cfg(feature = "serde")]
impl<T, const L: usize> Serialize for Chunk<T, L>
where T: Serialize {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    let nested_array = self::nested_array::from_inner_ref(&self.inner);
    <NestedArray<T, L>>::serialize(nested_array, serializer)
  }
}

#[cfg(feature = "serde")]
impl<'de, T, const L: usize> Deserialize<'de> for Chunk<T, L>
where T: Deserialize<'de> {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    <NestedArray<T, L>>::deserialize(deserializer).map(|nested_array| {
      Chunk { inner: self::nested_array::into_inner(nested_array) }
    })
  }
}

// This is necessary due to the array primitive's `Default` impl not actually being generic across all `N`.
fn new_inner<T, F: FnMut(LocalPos) -> T, const N: usize>(mut f: F) -> [[T; N]; N] {
  std::array::from_fn(|y| {
    std::array::from_fn(|x| {
      f([x, y])
    })
  })
}

fn from_nested_array<T, const N: usize>(array: [[T; N]; N]) -> Box<[T]> {
  let Some(len) = usize::checked_mul(N, N) else { panic!() };
  let array: Box<[[T; N]; N]> = Box::new(array);
  unsafe {
    // Convert the box into a pointer, then a wide pointer, then a wide box-pointer
    let array_ptr = Box::into_raw(array) as *mut T;
    let ptr = std::slice::from_raw_parts_mut(array_ptr, len) as *mut [T];
    Box::from_raw(ptr)
  }
}

fn from_nested_array_ref<T, const N: usize>(array: &[[T; N]; N]) -> &[T] {
  let Some(len) = usize::checked_mul(N, N) else { panic!() };
  unsafe {
    let ptr = array as *const [[T; N]; N] as *const T;
    std::slice::from_raw_parts(ptr, len)
  }
}

fn from_nested_array_mut<T, const N: usize>(array: &mut [[T; N]; N]) -> &mut [T] {
  let Some(len) = usize::checked_mul(N, N) else { panic!() };
  unsafe {
    let ptr = array as *mut [[T; N]; N] as *mut T;
    std::slice::from_raw_parts_mut(ptr, len)
  }
}

pub(crate) fn lerp_2d<T>(aa: T, ab: T, ba: T, bb: T, factor: Vector2<f32>) -> T
where T: Lerp<Output = T> {
  T::lerp(
    T::lerp(aa, ab, factor.y),
    T::lerp(ba, bb, factor.y),
    factor.x
  )
}

pub(crate) fn sample_2d<T, D, F>(pos: Vector2<f32>, mut f: F) -> T
where
  T: Lerp<Output = T>, F: FnMut(Vector2<D>) -> T,
  D: AsPrimitive<f32> + Eq, f32: AsPrimitive<D>
{
  let min = pos.map(f32::floor).cast::<D>();
  let max = pos.map(f32::ceil).cast::<D>();
  if min == max {
    return f(min);
  };

  let factor = pos.map(|v| v.rem_euclid(1.0));

  lerp_2d(
    f(Vector2::new(min.x, min.y)),
    f(Vector2::new(min.x, max.y)),
    f(Vector2::new(max.x, min.y)),
    f(Vector2::new(max.x, max.y)),
    factor
  )
}

pub(crate) fn try_sample_2d<T, D, F>(pos: Vector2<f32>, mut f: F) -> Option<T>
where
  T: Lerp<Output = T>, F: FnMut(Vector2<D>) -> Option<T>,
  D: AsPrimitive<f32> + Eq, f32: AsPrimitive<D>
{
  let min = pos.map(f32::floor).cast::<D>();
  let max = pos.map(f32::ceil).cast::<D>();
  if min == max {
    return f(min);
  };

  let factor = pos.map(|v| v.rem_euclid(1.0));

  Some(lerp_2d(
    f(Vector2::new(min.x, min.y))?,
    f(Vector2::new(min.x, max.y))?,
    f(Vector2::new(max.x, min.y))?,
    f(Vector2::new(max.x, max.y))?,
    factor
  ))
}
