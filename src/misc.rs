pub(crate) fn from_2nested_array<T, const N: usize>(array: [[T; N]; N]) -> Box<[T]> {
  let Some(len) = usize::checked_mul(N, N) else { panic!() };
  let array: Box<[[T; N]; N]> = Box::new(array);
  unsafe {
    // Convert the box into a pointer, then a wide pointer, then a wide box-pointer
    let array_ptr = Box::into_raw(array) as *mut T;
    let ptr = std::slice::from_raw_parts_mut(array_ptr, len) as *mut [T];
    Box::from_raw(ptr)
  }
}

pub(crate) fn from_2nested_array_ref<T, const N: usize>(array: &[[T; N]; N]) -> &[T] {
  let Some(len) = usize::checked_mul(N, N) else { panic!() };
  unsafe {
    let ptr = array as *const [[T; N]; N] as *const T;
    std::slice::from_raw_parts(ptr, len)
  }
}

pub(crate) fn from_2nested_array_mut<T, const N: usize>(array: &mut [[T; N]; N]) -> &mut [T] {
  let Some(len) = usize::checked_mul(N, N) else { panic!() };
  unsafe {
    let ptr = array as *mut [[T; N]; N] as *mut T;
    std::slice::from_raw_parts_mut(ptr, len)
  }
}

pub(crate) fn from_3nested_array<T, const N: usize>(array: [[[T; N]; N]; N]) -> Box<[T]> {
  let Some(len) = usize::checked_pow(N, 3) else { panic!() };
  let array: Box<[[[T; N]; N]; N]> = Box::new(array);
  unsafe {
    // Convert the box into a pointer, then a wide pointer, then a wide box-pointer
    let array_ptr = Box::into_raw(array) as *mut T;
    let ptr = std::slice::from_raw_parts_mut(array_ptr, len) as *mut [T];
    Box::from_raw(ptr)
  }
}

pub(crate) fn from_3nested_array_ref<T, const N: usize>(array: &[[[T; N]; N]; N]) -> &[T] {
  let Some(len) = usize::checked_pow(N, 3) else { panic!() };
  unsafe {
    let ptr = array as *const [[[T; N]; N]; N] as *const T;
    std::slice::from_raw_parts(ptr, len)
  }
}

pub(crate) fn from_3nested_array_mut<T, const N: usize>(array: &mut [[[T; N]; N]; N]) -> &mut [T] {
  let Some(len) = usize::checked_pow(N, 3) else { panic!() };
  unsafe {
    let ptr = array as *mut [[[T; N]; N]; N] as *mut T;
    std::slice::from_raw_parts_mut(ptr, len)
  }
}
