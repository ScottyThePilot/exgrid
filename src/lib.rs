#[cfg(feature = "multi-thread")]
pub extern crate rayon;
pub extern crate vek;

#[macro_use]
mod macros;
#[cfg(feature = "automata")]
pub mod automata;
pub mod chunk;
pub mod grid;

use vek::Vec2;

pub use crate::chunk::{Chunk, ChunkSparse};
pub use crate::grid::{ExGrid, ExGridSparse};

pub type GlobalPos = Vec2<i64>;
pub type ChunkPos = Vec2<i32>;
pub type LocalPos = Vec2<usize>;
