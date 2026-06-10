#[cfg(feature = "automata")]
pub mod automata;
pub mod chunk;
pub mod grid;

/// The position of a cell within a grid.
pub type GlobalPos = [i64; 2];
/// The position of a chunk within a grid.
pub type ChunkPos = [i32; 2];
/// The position of a cell within a chunk.
pub type LocalPos = [usize; 2];
