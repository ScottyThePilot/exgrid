macro_rules! impl_iterator_methods {
  () => {
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
      self.inner.next()
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where F: FnMut(B, Self::Item) -> B {
      self.inner.fold(init, f)
    }
  };
}

macro_rules! impl_iterator_methods_known_size {
  ($size:expr) => {
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
      ($size, Some($size))
    }

    #[inline]
    fn count(self) -> usize {
      $size
    }
  };
}

macro_rules! impl_double_ended_iterator_methods {
  () => {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
      self.inner.next_back()
    }

    #[inline]
    fn rfold<A, F>(self, init: A, f: F) -> A
    where F: FnMut(A, Self::Item) -> A {
      self.inner.rfold(init, f)
    }
  };
}

macro_rules! impl_exact_size_iterator_methods {
  ($size:expr) => {
    #[inline]
    fn len(&self) -> usize {
      $size
    }
  };
}

macro_rules! impl_iterator_known_size {
  ($Struct:ident, <'a, T, S>, $Item:ty, $size:expr) => {
    impl_iterator_known_size_no_double_ended!($Struct, <'a, T, S>, $Item, $size);

    impl<'a, T, const S: usize> DoubleEndedIterator for $Struct<'a, T, S> {
      impl_double_ended_iterator_methods!();
    }
  };
  ($Struct:ident, <T, S>, $Item:ty, $size:expr) => {
    impl_iterator_known_size_no_double_ended!($Struct, <T, S>, $Item, $size);

    impl<T, const S: usize> DoubleEndedIterator for $Struct<T, S> {
      impl_double_ended_iterator_methods!();
    }
  };
}

macro_rules! impl_iterator_known_size_no_double_ended {
  ($Struct:ident, <'a, T, S>, $Item:ty, $size:expr) => {
    impl<'a, T, const S: usize> Iterator for $Struct<'a, T, S> {
      type Item = $Item;

      impl_iterator_methods!();
      impl_iterator_methods_known_size!($size);
    }

    impl<'a, T, const S: usize> ExactSizeIterator for $Struct<'a, T, S> {
      impl_exact_size_iterator_methods!($size);
    }

    impl<'a, T, const S: usize> FusedIterator for $Struct<'a, T, S> {}
  };
  ($Struct:ident, <T, S>, $Item:ty, $size:expr) => {
    impl<T, const S: usize> Iterator for $Struct<T, S> {
      type Item = $Item;

      impl_iterator_methods!();
      impl_iterator_methods_known_size!($size);
    }

    impl<T, const S: usize> ExactSizeIterator for $Struct<T, S> {
      impl_exact_size_iterator_methods!($size);
    }

    impl<T, const S: usize> FusedIterator for $Struct<T, S> {}
  };
}

macro_rules! impl_iterator {
  ($Struct:ident, <'a, T, S>, $Item:ty $(, $size_hint:expr)?) => {
    impl_iterator_no_double_ended!($Struct, <'a, T, S>, $Item $(, $size_hint)?);

    impl<'a, T, const S: usize> DoubleEndedIterator for $Struct<'a, T, S> {
      impl_double_ended_iterator_methods!();
    }
  };
  ($Struct:ident, <T, S>, $Item:ty $(, $size_hint:expr)?) => {
    impl_iterator_no_double_ended!($Struct, <T, S>, $Item $(, $size_hint)?);

    impl<T, const S: usize> DoubleEndedIterator for $Struct<T, S> {
      impl_double_ended_iterator_methods!();
    }
  };
}

macro_rules! impl_iterator_no_double_ended {
  ($Struct:ident, <'a, T, S>, $Item:ty $(, $size_hint:expr)?) => {
    impl<'a, T, const S: usize> Iterator for $Struct<'a, T, S> {
      type Item = $Item;

      impl_iterator_methods!();

      $(#[inline] fn size_hint(&self) -> (usize, Option<usize>) {
        $size_hint
      })?
    }

    impl<'a, T, const S: usize> FusedIterator for $Struct<'a, T, S> {}
  };
  ($Struct:ident, <T, S>, $Item:ty $(, $size_hint:expr)?) => {
    impl<T, const S: usize> Iterator for $Struct<T, S> {
      type Item = $Item;

      impl_iterator_methods!();

      $(#[inline] fn size_hint(&self) -> (usize, Option<usize>) {
        $size_hint
      })?
    }

    impl<T, const S: usize> FusedIterator for $Struct<T, S> {}
  };
}

#[cfg(feature = "multi-thread")]
macro_rules! impl_par_iterator_methods {
  () => {
    #[inline]
    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where C: UnindexedConsumer<Self::Item> {
      self.inner.drive_unindexed(consumer)
    }

    #[inline]
    fn opt_len(&self) -> Option<usize> {
      self.inner.opt_len()
    }
  };
}

#[cfg(feature = "multi-thread")]
macro_rules! impl_par_indexed_iterator_methods {
  () => {
    #[inline]
    fn drive<C>(self, consumer: C) -> C::Result
    where C: Consumer<Self::Item> {
      self.inner.drive(consumer)
    }

    #[inline]
    fn len(&self) -> usize {
      self.inner.len()
    }

    #[inline]
    fn with_producer<CB>(self, callback: CB) -> CB::Output
    where CB: ProducerCallback<Self::Item> {
      self.inner.with_producer(callback)
    }
  };
}

#[cfg(feature = "multi-thread")]
macro_rules! impl_par_iterator {
  ($Struct:ident, <'data, T: $($Bound:ident +)? 'data, S>, $Item:ty) => {
    impl<'data, T: $($Bound +)? 'data, const S: usize> ParallelIterator for $Struct<'data, T, S> {
      type Item = $Item;

      impl_par_iterator_methods!();
    }
  };
  ($Struct:ident, <T $(: $Bound:ident)?, S>, $Item:ty) => {
    impl<T: Send, const S: usize> ParallelIterator for $Struct<T, S> {
      type Item = $Item;

      impl_par_iterator_methods!();
    }
  };
}

#[cfg(feature = "multi-thread")]
macro_rules! impl_par_iterator_indexed {
  ($Struct:ident, <'data, T: $($Bound:ident +)? 'data, S>, $Item:ty) => {
    impl_par_iterator!($Struct, <'data, T $(: $Bound +)? 'data, S>, $Item);

    impl<'data, T: $($Bound +)? 'data, const S: usize> IndexedParallelIterator for $Struct<'data, T, S> {
      impl_par_indexed_iterator_methods!();
    }
  };
  ($Struct:ident, <T $(: $Bound:ident)?, S>, $Item:ty) => {
    impl_par_iterator!($Struct, <T $(: $Bound)?, S>, $Item);

    impl<T: Send, const S: usize> IndexedParallelIterator for $Struct<T, S> {
      impl_par_indexed_iterator_methods!();
    }
  };
}
