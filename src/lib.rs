#![warn(
  absolute_paths_not_starting_with_crate,
  redundant_imports,
  redundant_lifetimes,
  future_incompatible,
  deprecated_in_future,
  missing_copy_implementations,
  missing_debug_implementations,
  unnameable_types,
  unreachable_pub
)]

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

pub use crate::vector::Lerp;
