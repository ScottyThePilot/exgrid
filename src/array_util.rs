use arrayvec::ArrayVec;

pub trait ArrayExt<T, const N: usize>: Sized {
  fn try_map_opt<U>(self, f: impl FnMut(T) -> Option<U>) -> Option<[U; N]>;
  fn try_map_res<U, E>(self, f: impl FnMut(T) -> Result<U, E>) -> Result<[U; N], E>;
}

impl<T, const N: usize> ArrayExt<T, N> for [T; N] {
  fn try_map_opt<U>(self, mut f: impl FnMut(T) -> Option<U>) -> Option<[U; N]> {
    self.try_map_res(|value| f(value).ok_or(())).ok()
  }

  fn try_map_res<U, E>(self, mut f: impl FnMut(T) -> Result<U, E>) -> Result<[U; N], E> {
    let mut out = ArrayVec::new();
    for value in self {
      let value_out = f(value)?;
      unsafe { out.push_unchecked(value_out) };
    };

    Ok(unsafe { out.into_inner_unchecked() })
  }
}
