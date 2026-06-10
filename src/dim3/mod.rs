pub mod chunk;
pub mod grid;

/// The position of a cell within a grid.
pub type GlobalPos = [i64; 3];
/// The position of a chunk within a grid.
pub type ChunkPos = [i32; 3];
/// The position of a cell within a chunk.
pub type LocalPos = [usize; 3];
