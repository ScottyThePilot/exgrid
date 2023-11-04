use serde_big_array::Array;
use std::mem::ManuallyDrop;

pub(crate) type Array2Nested<T, const N: usize> = Array<Array<T, N>, N>;

pub(crate) union Array2NestedRepr<T, const N: usize> {
  inner: ManuallyDrop<[[T; N]; N]>,
  nested_array: ManuallyDrop<Array2Nested<T, N>>
}

impl<T, const N: usize> Array2NestedRepr<T, N> {
  pub(crate) const fn from_array_ref(inner: &[[T; N]; N]) -> &Array2Nested<T, N> {
    // SAFETY: Array<Array<T, N>, N> is #[repr(transparent)] and thus
    // has the same memory layout as its inner type, [[T; N]; N]
    unsafe {
      &*(inner as *const [[T; N]; N] as *const Array2Nested<T, N>)
    }
  }

  pub(crate) const fn into_array(nested_array: Array2Nested<T, N>) -> [[T; N]; N] {
    // SAFETY: Array<Array<T, N>, N> is #[repr(transparent)] and thus
    // has the same memory layout as its inner type, [[T; N]; N]
    unsafe {
      ManuallyDrop::into_inner(Array2NestedRepr {
        nested_array: ManuallyDrop::new(nested_array)
      }.inner)
    }
  }
}

pub(crate) type Array3Nested<T, const N: usize> = Array<Array<Array<T, N>, N>, N>;

pub(crate) union Array3NestedRepr<T, const N: usize> {
  inner: ManuallyDrop<[[[T; N]; N]; N]>,
  nested_array: ManuallyDrop<Array3Nested<T, N>>
}

impl<T, const N: usize> Array3NestedRepr<T, N> {
  pub(crate) const fn from_array_ref(inner: &[[[T; N]; N]; N]) -> &Array3Nested<T, N> {
    // SAFETY: Array<Array<Array<T, N>, N>, N> is #[repr(transparent)] and thus
    // has the same memory layout as its inner type, [[[T; N]; N]; N]
    unsafe {
      &*(inner as *const [[[T; N]; N]; N] as *const Array3Nested<T, N>)
    }
  }

  pub(crate) const fn into_array(nested_array: Array3Nested<T, N>) -> [[[T; N]; N]; N] {
    // SAFETY: Array<Array<Array<T, N>, N>, N> is #[repr(transparent)] and thus
    // has the same memory layout as its inner type, [[[T; N]; N]; N]
    unsafe {
      ManuallyDrop::into_inner(Array3NestedRepr {
        nested_array: ManuallyDrop::new(nested_array)
      }.inner)
    }
  }
}
