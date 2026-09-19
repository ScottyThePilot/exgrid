#[cfg(feature = "multi-thread")]
pub extern crate rayon;

#[macro_use]
mod macros;
mod array_util;
pub mod dim2;
pub mod dim3;
#[cfg(feature = "serde")]
mod nested_array;
mod vector;

#[deprecated]
pub type Chunk<T, const S: usize> = crate::dim2::chunk::Chunk<T, S>;
#[deprecated]
pub type ChunkSparse<T, const S: usize> = crate::dim2::chunk::ChunkSparse<T, S>;
#[deprecated]
pub type ExGrid<T, const S: usize, H = std::collections::hash_map::RandomState> = crate::dim2::grid::ExGrid<T, S, H>;
#[deprecated]
pub type ExGridSparse<T, const S: usize, H = std::collections::hash_map::RandomState> = crate::dim2::grid::ExGridSparse<T, S, H>;

pub use crate::vector::Lerp;

#[deprecated]
pub type GlobalPos = [i64; 2];
#[deprecated]
pub type ChunkPos = [i32; 2];
#[deprecated]
pub type LocalPos = [usize; 2];
