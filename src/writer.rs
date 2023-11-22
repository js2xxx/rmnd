use core::{mem, ops::AddAssign};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Writer<T, W>(pub T, pub W);

impl<T, W> Writer<T, W> {
    pub fn bind<U, H>(self, h: H) -> Writer<U, W>
    where
        W: Append,
        H: FnOnce(T) -> Writer<U, W>,
    {
        let Writer(t, mut w) = self;
        let Writer(u, mut wo) = h(t);
        w.append(&mut wo);
        Writer(u, w)
    }

    pub fn listen(self) -> Writer<(T, W), W>
    where
        W: Clone,
    {
        let Writer(t, w) = self;
        Writer((t, w.clone()), w)
    }

    pub fn listens<U, F: FnOnce(W) -> (U, W)>(self, f: F) -> Writer<(T, U), W> {
        let Writer(t, w) = self;
        let (u, w) = f(w);
        Writer((t, u), w)
    }

    pub fn censor<F: FnOnce(W) -> W>(self, f: F) -> Writer<T, W> {
        let Writer(t, w) = self;
        Writer(t, f(w))
    }
}

impl<T, W, G: FnOnce(W) -> W> Writer<(T, G), W> {
    pub fn pass(self) -> Writer<T, W> {
        let Writer((t, g), w) = self;
        Writer(t, g(w))
    }
}

pub fn value<T, W: Default>(value: T) -> Writer<T, W> {
    Writer(value, W::default())
}

pub fn write<W>(w: W) -> Writer<(), W> {
    Writer((), w)
}

#[macro_export]
macro_rules! writer {
    {@ $e:expr} => ($e);
    {$e:expr} => ($crate::writer::value($e));
    {let $v:pat = @ $e:expr $(=> $ty:ty)?; $($t:tt)*} => {{
        let closure = move |$v $(:$ty)?| writer!($($t)*);
        $e .bind(closure)
    }};
    {let $v:pat = $e:expr $(=> $ty:ty)?; $($t:tt)*} => {{
        let closure = move |$v $(:$ty)?| writer!($($t)*);
        $crate::writer::value($e).bind(closure)
    }};
}

#[cfg(test)]
mod tests {
    use crate::writer::Writer;

    use super::{write, Count};
    use std::format;

    #[test]
    fn basic() {
        let w = writer! {
            let x = 1;
            let _ = @write(format!("{x}\n"));
            let y = x + 2;
            let _ = @write(format!("{y}\n"));
            x + y
        };
        let Writer(v, w) = w;
        assert_eq!(v, 4);
        assert_eq!(w, "1\n3\n");
    }

    #[test]
    fn count() {
        let w = writer! {
            let x = 1 => i32;
            let _ = @write(Count(x));
            let y = x + 2;
            let _ = @write(Count(y));
            x + y
        };
        let Writer(v, Count(w)) = w;
        assert_eq!(v, w);
    }
}
