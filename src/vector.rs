use std::ops::{Add, Sub, Mul, Div, Rem, Neg};

use num_traits::AsPrimitive;

macro_rules! operator_impl {
  ($Vector:ident, $Op:ident, $op:ident, $SIZE:literal) => {
    impl<T: $Op> $Op<[T; $SIZE]> for $Vector<T> {
      type Output = $Vector<<T as $Op>::Output>;

      #[inline]
      fn $op(self, rhs: [T; $SIZE]) -> Self::Output {
        self.zip_map(Self::from_array(rhs), T::$op)
      }
    }

    impl<T: $Op> $Op for $Vector<T> {
      type Output = $Vector<<T as $Op>::Output>;

      #[inline]
      fn $op(self, rhs: Self) -> Self::Output {
        self.zip_map(rhs, T::$op)
      }
    }

    impl<T: $Op + Clone> $Op<T> for $Vector<T> {
      type Output = $Vector<<T as $Op>::Output>;

      #[inline]
      fn $op(self, rhs: T) -> Self::Output {
        self.map(|field| T::$op(field, rhs.clone()))
      }
    }
  };
}

macro_rules! vector_impl {
  ($vis:vis struct $Vector:ident { $($f:ident),* }: $SIZE:literal) => {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    $vis struct $Vector<T> { $(pub $f: T),* }

    impl<T> $Vector<T> {
      #[inline]
      pub const fn new($($f: T),*) -> Self {
        $Vector { $($f),* }
      }

      #[inline]
      pub fn cast<D>(self) -> $Vector<D>
      where T: AsPrimitive<D>, D: 'static + Copy {
        $Vector { $($f: self.$f.as_()),* }
      }

      #[inline]
      pub fn map<U, F>(self, mut f: F) -> $Vector<U>
      where F: FnMut(T) -> U {
        $Vector { $($f: f(self.$f)),* }
      }

      #[inline]
      pub fn zip_map<U, V, F>(self, other: $Vector<U>, mut f: F) -> $Vector<V>
      where F: FnMut(T, U) -> V {
        $Vector { $($f: f(self.$f, other.$f)),* }
      }

      #[inline]
      pub fn from_array(array: [T; $SIZE]) -> Self {
        let [$($f),*] = array;
        $Vector { $($f),* }
      }

      #[inline]
      pub fn into_array(self) -> [T; $SIZE] {
        [$(self.$f),*]
      }
    }

    operator_impl!($Vector, Add, add, $SIZE);
    operator_impl!($Vector, Sub, sub, $SIZE);
    operator_impl!($Vector, Mul, mul, $SIZE);
    operator_impl!($Vector, Div, div, $SIZE);
    operator_impl!($Vector, Rem, rem, $SIZE);

    impl<T: Neg> Neg for $Vector<T> {
      type Output = $Vector<<T as Neg>::Output>;

      fn neg(self) -> Self::Output {
        self.map(T::neg)
      }
    }

    impl<T> From<$Vector<T>> for [T; $SIZE] {
      #[inline]
      fn from(value: $Vector<T>) -> Self {
        value.into_array()
      }
    }

    impl<T> From<[T; $SIZE]> for $Vector<T> {
      #[inline]
      fn from(value: [T; $SIZE]) -> Self {
        $Vector::from_array(value)
      }
    }
  };
}

vector_impl!(pub(crate) struct Vector2 { x, y }: 2);

pub trait Lerp<Factor = f32> {
  type Output;

  fn lerp(from: Self, to: Self, factor: Factor) -> Self::Output;
}

impl Lerp<f32> for f32 {
  type Output = f32;

  fn lerp(from: Self, to: Self, factor: f32) -> Self::Output {
    from.mul_add(1.0 - factor, to * factor)
  }
}

impl Lerp<f64> for f64 {
  type Output = f64;

  fn lerp(from: Self, to: Self, factor: f64) -> Self::Output {
    from.mul_add(1.0 - factor, to * factor)
  }
}
