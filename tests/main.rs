extern crate exgrid;

use exgrid::grid::*;

use rand::Rng;

macro_rules! perform_test_g {
  ($function:ident) => {
    perform_test_g!($function, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2)
  };
  ($function:ident, $($G:literal),+) => {
    $( $function ::<$G>(); )+
  };
}

#[test]
fn test_translate() {
  perform_test_g!(test_translate_g);
}

fn test_translate_g<const S: usize>() {
  for c1 in random_positions() {
    let d1 = decompose::<S>(c1);
    let c2 = compose::<S>(d1.0, d1.1);
    assert_eq!(c1, c2);
  };
}

#[test]
fn test_grid_basics() {
  perform_test_g!(test_grid_basics_g);
}

fn test_grid_basics_g<const S: usize>()
where [[u32; S]; S]: Default {
  let mut grid = ExGrid::<u32, S>::new();
  for ([x, y], value) in random_elements() {
    *grid.get_mut_default(x, y) = value;
    assert_eq!(grid.get(x, y), Some(&value));
  };



  for ([x, y], &value) in grid.cells() {
    assert_eq!(grid.get(x, y), Some(&value), "{x}, {y}");
  };

  for ([x, y], &mut value) in grid.clone().cells_mut() {
    assert_eq!(grid.get(x, y), Some(&value), "{x}, {y}");
  };

  for ([x, y], value) in grid.clone().into_cells() {
    assert_eq!(grid.get(x, y), Some(&value), "{x}, {y}");
  };
}

#[test]
fn test_grid_sparse_basics() {
  perform_test_g!(test_grid_sparse_basics_g);
}

fn test_grid_sparse_basics_g<const S: usize>()
where [[Option<u32>; S]; S]: Default {
  let mut grid = ExGridSparse::<u32, S>::new();
  for ([x, y], value) in random_elements() {
    grid.insert(x, y, value);
    assert_eq!(grid.get(x, y), Some(&value));
  };

  for ([x, y], &value) in grid.cells() {
    assert_eq!(grid.get(x, y), Some(&value), "{x}, {y}");
  };

  for ([x, y], &mut value) in grid.clone().cells_mut() {
    assert_eq!(grid.get(x, y), Some(&value), "{x}, {y}");
  };

  for ([x, y], value) in grid.clone().into_cells() {
    assert_eq!(grid.get(x, y), Some(&value), "{x}, {y}");
  };
}

fn random_position(rng: &mut impl Rng) -> [isize; 2] {
  let r = (i32::MIN as isize)..=(i32::MAX as isize);
  [rng.gen_range(r.clone()), rng.gen_range(r)]
}

fn random_positions() -> impl Iterator<Item = [isize; 2]> {
  let mut rng = rand::thread_rng();
  let count = rng.gen_range(16..32);
  std::iter::repeat_with(move || {
    random_position(&mut rng)
  }).take(count)
}

fn random_elements() -> impl Iterator<Item = ([isize; 2], u32)> {
  let mut rng = rand::thread_rng();
  let count = rng.gen_range(16..32);
  std::iter::repeat_with(move || {
    (random_position(&mut rng), rng.gen::<u32>())
  }).take(count)
}
