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
vector_impl!(pub(crate) struct Vector3 { x, y, z }: 3);

pub trait Lerp<Factor = f32> {
  type Output;

  fn lerp(values: [Self; 2], factor: Factor) -> Self::Output where Self: Sized;
}

impl Lerp<f32> for f32 {
  type Output = f32;

  fn lerp([from, to]: [Self; 2], factor: f32) -> Self::Output {
    from.mul_add(1.0 - factor, to * factor)
  }
}

impl Lerp<f64> for f64 {
  type Output = f64;

  fn lerp([from, to]: [Self; 2], factor: f64) -> Self::Output {
    from.mul_add(1.0 - factor, to * factor)
  }
}

impl<T, Factor, const N: usize> Lerp<[Factor; N]> for T
where T: Lerp<Factor::Compound>, Factor: IntoCompound<N> {
  type Output = T::Output;

  fn lerp(values: [Self; 2], factor: [Factor; N]) -> Self::Output where Self: Sized {
    Lerp::lerp(values, Factor::into_compound(factor))
  }
}

impl<T, FactorInner, FactorOuter> Lerp<Compound<FactorInner, FactorOuter>> for [T; 2]
where T: Lerp<FactorInner>, T::Output: Lerp<FactorOuter>, FactorInner: Copy {
  type Output = <T::Output as Lerp<FactorOuter>>::Output;

  fn lerp(values: [Self; 2], factor: Compound<FactorInner, FactorOuter>) -> Self::Output {
    Lerp::lerp(values.map(|values| {
      Lerp::lerp(values, factor.inner)
    }), factor.outer)
  }
}



#[derive(Debug, Clone, Copy)]
pub struct Compound<FactorInner, FactorOuter> {
  inner: FactorInner,
  outer: FactorOuter
}

impl<FactorInner, FactorOuter> Compound<FactorInner, FactorOuter> {
  const fn new(inner: FactorInner, outer: FactorOuter) -> Self {
    Compound { inner, outer }
  }
}

pub trait IntoCompound<const N: usize> {
  type Compound;

  fn into_compound(values: [Self; N]) -> Self::Compound where Self: Sized;
}

impl<T> IntoCompound<2> for T {
  type Compound = Compound<T, T>;

  fn into_compound([t0, t1]: [Self; 2]) -> Self::Compound {
    Compound::new(t1, t0)
  }
}

impl<T> IntoCompound<3> for T {
  type Compound = Compound<Compound<T, T>, T>;

  fn into_compound([t0, t1, t2]: [Self; 3]) -> Self::Compound {
    Compound::new(Compound::new(t2, t1), t0)
  }
}

impl<T> IntoCompound<4> for T {
  type Compound = Compound<Compound<Compound<T, T>, T>, T>;

  fn into_compound([t0, t1, t2, t3]: [Self; 4]) -> Self::Compound {
    Compound::new(Compound::new(Compound::new(t3, t2), t1), t0)
  }
}



#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_2d_lerp() {
    let mut matrix: [[f32; 2]; 2] = Default::default();

    matrix[0][0] = 25.0;
    matrix[1][0] = 10.0;
    matrix[0][1] = 13.7;
    matrix[1][1] = 40.2;

    assert_eq!(Lerp::lerp(matrix, [0.0, 0.0]), 25.0, "at [0, 0]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 0.0]), 10.0, "at [1, 0]");
    assert_eq!(Lerp::lerp(matrix, [0.0, 1.0]), 13.7, "at [0, 1]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 1.0]), 40.2, "at [1, 1]");
  }

  #[test]
  fn test_3d_lerp() {
    let mut matrix: [[[f32; 2]; 2]; 2] = Default::default();

    matrix[0][0][0] = 25.0;
    matrix[1][0][0] = 10.0;
    matrix[0][1][0] = 13.7;
    matrix[1][1][0] = 40.2;
    matrix[0][0][1] = 20.6;
    matrix[1][0][1] = 40.9;
    matrix[0][1][1] = 84.4;
    matrix[1][1][1] = 11.1;

    assert_eq!(Lerp::lerp(matrix, [0.0, 0.0, 0.0]), 25.0, "at [0, 0, 0]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 0.0, 0.0]), 10.0, "at [1, 0, 0]");
    assert_eq!(Lerp::lerp(matrix, [0.0, 1.0, 0.0]), 13.7, "at [0, 1, 0]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 1.0, 0.0]), 40.2, "at [1, 1, 0]");
    assert_eq!(Lerp::lerp(matrix, [0.0, 0.0, 1.0]), 20.6, "at [0, 0, 1]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 0.0, 1.0]), 40.9, "at [1, 0, 1]");
    assert_eq!(Lerp::lerp(matrix, [0.0, 1.0, 1.0]), 84.4, "at [0, 1, 1]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 1.0, 1.0]), 11.1, "at [1, 1, 1]");
  }

  #[test]
  fn test_4d_lerp() {
    let mut matrix: [[[[f32; 2]; 2]; 2]; 2] = Default::default();

    matrix[0][0][0][0] = 25.0;
    matrix[1][0][0][0] = 10.0;
    matrix[0][1][0][0] = 13.7;
    matrix[1][1][0][0] = 40.2;
    matrix[0][0][1][0] = 20.6;
    matrix[1][0][1][0] = 40.9;
    matrix[0][1][1][0] = 84.4;
    matrix[1][1][1][0] = 11.1;
    matrix[0][0][0][1] = 65.7;
    matrix[1][0][0][1] = 17.0;
    matrix[0][1][0][1] = 28.8;
    matrix[1][1][0][1] = 99.5;
    matrix[0][0][1][1] = 34.6;
    matrix[1][0][1][1] = 94.2;
    matrix[0][1][1][1] = 22.9;
    matrix[1][1][1][1] = 20.2;

    assert_eq!(Lerp::lerp(matrix, [0.0, 0.0, 0.0, 0.0]), 25.0, "at [0, 0, 0, 0]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 0.0, 0.0, 0.0]), 10.0, "at [1, 0, 0, 0]");
    assert_eq!(Lerp::lerp(matrix, [0.0, 1.0, 0.0, 0.0]), 13.7, "at [0, 1, 0, 0]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 1.0, 0.0, 0.0]), 40.2, "at [1, 1, 0, 0]");
    assert_eq!(Lerp::lerp(matrix, [0.0, 0.0, 1.0, 0.0]), 20.6, "at [0, 0, 1, 0]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 0.0, 1.0, 0.0]), 40.9, "at [1, 0, 1, 0]");
    assert_eq!(Lerp::lerp(matrix, [0.0, 1.0, 1.0, 0.0]), 84.4, "at [0, 1, 1, 0]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 1.0, 1.0, 0.0]), 11.1, "at [1, 1, 1, 0]");
    assert_eq!(Lerp::lerp(matrix, [0.0, 0.0, 0.0, 1.0]), 65.7, "at [0, 0, 0, 1]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 0.0, 0.0, 1.0]), 17.0, "at [1, 0, 0, 1]");
    assert_eq!(Lerp::lerp(matrix, [0.0, 1.0, 0.0, 1.0]), 28.8, "at [0, 1, 0, 1]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 1.0, 0.0, 1.0]), 99.5, "at [1, 1, 0, 1]");
    assert_eq!(Lerp::lerp(matrix, [0.0, 0.0, 1.0, 1.0]), 34.6, "at [0, 0, 1, 1]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 0.0, 1.0, 1.0]), 94.2, "at [1, 0, 1, 1]");
    assert_eq!(Lerp::lerp(matrix, [0.0, 1.0, 1.0, 1.0]), 22.9, "at [0, 1, 1, 1]");
    assert_eq!(Lerp::lerp(matrix, [1.0, 1.0, 1.0, 1.0]), 20.2, "at [1, 1, 1, 1]");
  }
}
