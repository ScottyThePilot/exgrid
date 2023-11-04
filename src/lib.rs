#[cfg(feature = "multi-thread")]
pub extern crate rayon;

#[macro_use]
mod macros;
pub mod dim2;
pub mod dim3;
mod misc;
#[cfg(feature = "serde")]
mod nested_array;
mod vector;

pub use crate::dim2::chunk::{Chunk, ChunkSparse};
pub use crate::dim2::grid::{ExGrid, ExGridSparse};
pub use crate::vector::Lerp;

pub type GlobalPos = [i64; 2];
pub type ChunkPos = [i32; 2];
pub type LocalPos = [usize; 2];
