use super::{Chunk, ChunkSparse};

use rayon::iter::{
  IndexedParallelIterator,
  IntoParallelIterator,
  ParallelIterator,
  Flatten
};
use rayon::iter::plumbing::{
  Consumer,
  ProducerCallback,
  UnindexedConsumer
};



#[repr(transparent)]
pub struct ChunkSparseIterPar<'data, T: Sync, const S: usize> {
  inner: Flatten<ChunkIterPar<'data, Option<T>, S>>
}

impl<'data, T: Sync, const S: usize> ChunkSparseIterPar<'data, T, S> {
  pub(crate) fn new(chunk: &'data ChunkSparse<T, S>) -> Self {
    let inner = ChunkIterPar::new(&chunk.inner).flatten();
    ChunkSparseIterPar { inner }
  }
}

impl_par_iterator!(ChunkSparseIterPar, <'data, T: Sync + 'data, S>, &'data T);

#[repr(transparent)]
pub struct ChunkSparseIterMutPar<'data, T: Send, const S: usize> {
  inner: Flatten<ChunkIterMutPar<'data, Option<T>, S>>
}

impl<'data, T: Send, const S: usize> ChunkSparseIterMutPar<'data, T, S> {
  pub(crate) fn new(chunk: &'data mut ChunkSparse<T, S>) -> Self {
    let inner = ChunkIterMutPar::new(&mut chunk.inner).flatten();
    ChunkSparseIterMutPar { inner }
  }
}

impl_par_iterator!(ChunkSparseIterMutPar, <'data, T: Send + 'data, S>, &'data mut T);

#[repr(transparent)]
pub struct ChunkSparseIntoIterPar<T: Send, const S: usize> {
  inner: Flatten<ChunkIntoIterPar<Option<T>, S>>
}

impl<T: Send, const S: usize> ChunkSparseIntoIterPar<T, S> {
  pub(crate) fn new(chunk: ChunkSparse<T, S>) -> Self {
    let inner = ChunkIntoIterPar::new(chunk.inner).flatten();
    ChunkSparseIntoIterPar { inner }
  }
}

impl_par_iterator!(ChunkSparseIntoIterPar, <T: Send, S>, T);



#[repr(transparent)]
pub struct ChunkIterPar<'data, T: Sync, const S: usize> {
  inner: <&'data [T] as IntoParallelIterator>::Iter
}

impl<'data, T: Sync, const S: usize> ChunkIterPar<'data, T, S> {
  pub(crate) fn new(chunk: &'data Chunk<T, S>) -> Self {
    let inner = super::cast_nested_array_ref(&chunk.inner).into_par_iter();
    ChunkIterPar { inner }
  }
}

impl_par_iterator_indexed!(ChunkIterPar, <'data, T: Sync + 'data, S>, &'data T);

#[repr(transparent)]
pub struct ChunkIterMutPar<'data, T: Send, const S: usize> {
  inner: <&'data mut [T] as IntoParallelIterator>::Iter
}

impl<'data, T: Send, const S: usize> ChunkIterMutPar<'data, T, S> {
  pub(crate) fn new(chunk: &'data mut Chunk<T, S>) -> Self {
    let inner = super::cast_nested_array_mut(&mut chunk.inner).into_par_iter();
    ChunkIterMutPar { inner }
  }
}

impl_par_iterator_indexed!(ChunkIterMutPar, <'data, T: Send + 'data, S>, &'data mut T);

#[repr(transparent)]
pub struct ChunkIntoIterPar<T: Send, const S: usize> {
  inner: <Vec<T> as IntoParallelIterator>::Iter
}

impl<T: Send, const S: usize> ChunkIntoIterPar<T, S> {
  pub(crate) fn new(chunk: Chunk<T, S>) -> Self {
    let inner = Vec::from(super::cast_nested_array(chunk.inner)).into_par_iter();
    ChunkIntoIterPar { inner }
  }
}

impl_par_iterator_indexed!(ChunkIntoIterPar, <T: Send, S>, T);
