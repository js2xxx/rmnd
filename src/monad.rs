use core::iter::FlatMap;
use core::{mem, ops::AddAssign};

pub trait Monad: IntoIterator {
    fn bind<U, F>(self, f: F) -> FlatMap<Self::IntoIter, U, F>
    where
        F: Fn(Self::Item) -> U,
        U: IntoIterator,
        Self: Sized,
    {
        self.into_iter().flat_map(f)
    }
}

impl<R> Monad for R where R: IntoIterator {}

pub use Some as value;

pub trait Append {
    fn append(&mut self, other: &mut Self);
}

#[cfg(any(test, feature = "alloc"))]
impl Append for alloc::string::String {
    fn append(&mut self, other: &mut Self) {
        self.push_str(other);
    }
}

#[cfg(any(test, feature = "alloc"))]
impl<T> Append for alloc::vec::Vec<T> {
    fn append(&mut self, other: &mut Self) {
        self.append(other)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Count<T>(pub T);

impl<T: AddAssign<T> + Default> Append for Count<T> {
    fn append(&mut self, other: &mut Self) {
        let other = mem::take(other);
        self.0 += other.0;
    }
}

#[macro_export]
macro_rules! monad {
    {@$e:expr} => ($e);
    {@_($b:expr) $e:expr} => (($b).then(|| $e));
    {$e:expr} => ($crate::monad::value($e));
    {let $v:pat = @ $e:expr $(=> $ty:ty)?; $($t:tt)*} => {{
        let closure = move |$v $(:$ty)?| $crate::monad!($($t)*);
        $e .bind(closure)
    }};
    {let $v:pat = @_($b:expr) $e:expr $(=> $ty:ty)?; $($t:tt)*} => {{
        let closure = move |$v $(:$ty)?| $crate::monad!($($t)*);
        ($b).then(|| $e).bind(closure)
    }};
    {@_($b:expr); $($t:tt)*} => {{
        let closure = move |_| $crate::monad!($($t)*);
        ($b).then_some(()).bind(closure)
    }};
    {let $v:pat = $e:expr $(=> $ty:ty)?; $($t:tt)*} => {{
        let closure = move |$v $(:$ty)?| $crate::monad!($($t)*);
        $crate::monad::value($e).bind(closure)
    }};
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use quickcheck_macros::quickcheck;

    use crate::monad::Monad;

    #[quickcheck]
    fn basic(input: Vec<i16>) {
        let m = monad! {
            let &item = @&input => &i16;
            @_(item < 4) (item as i32) * 2
        };
        let i = input
            .iter()
            .filter_map(|&item| (item < 4).then_some((item as i32) * 2));
        assert_eq!(m.collect::<Vec<_>>(), i.collect::<Vec<_>>());
    }
}
