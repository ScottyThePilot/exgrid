use serde_big_array::Array;
use std::mem::ManuallyDrop;

pub const fn from_inner_ref<T, const N: usize>(inner: &[[T; N]; N]) -> &NestedArray<T, N> {
  // SAFETY: Array<Array<T, N>, N> is #[repr(transparent)] and thus
  // has the same memory layout as its inner type, [[T; N]; N]
  unsafe {
    &*(inner as *const [[T; N]; N] as *const NestedArray<T, N>)
  }
}

pub const fn into_inner<T, const N: usize>(nested_array: NestedArray<T, N>) -> [[T; N]; N] {
  // SAFETY: Array<Array<T, N>, N> is #[repr(transparent)] and thus
  // has the same memory layout as its inner type, [[T; N]; N]
  unsafe {
    ManuallyDrop::into_inner(NestedArrayRepr {
      nested_array: ManuallyDrop::new(nested_array)
    }.inner)
  }
}

pub type NestedArray<T, const N: usize> = Array<Array<T, N>, N>;

union NestedArrayRepr<T, const N: usize> {
  inner: ManuallyDrop<[[T; N]; N]>,
  nested_array: ManuallyDrop<NestedArray<T, N>>
}
