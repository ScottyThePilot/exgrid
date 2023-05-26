#[cfg(feature = "multi-thread")]
pub extern crate rayon;

#[macro_use]
mod macros;
#[cfg(feature = "automata")]
pub mod automata;
pub mod chunk;
pub mod grid;
mod vector;

pub use crate::chunk::{Chunk, ChunkSparse};
pub use crate::grid::{ExGrid, ExGridSparse};
pub use crate::vector::Lerp;

pub type GlobalPos = [i64; 2];
pub type ChunkPos = [i32; 2];
pub type LocalPos = [usize; 2];
