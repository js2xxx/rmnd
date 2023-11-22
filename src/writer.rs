use crate::monad::Append;

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
        let closure = move |$v $(:$ty)?| $crate::writer!($($t)*);
        $e .bind(closure)
    }};
    {let $v:pat = $e:expr $(=> $ty:ty)?; $($t:tt)*} => {{
        let closure = move |$v $(:$ty)?| $crate::writer!($($t)*);
        $crate::writer::value($e).bind(closure)
    }};
}

#[cfg(test)]
mod tests {
    use quickcheck_macros::quickcheck;

    use crate::{monad::Count, writer::Writer};

    use super::write;
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

    #[quickcheck]
    fn count(input: i16) {
        let w = writer! {
            let x = input as i32 => i32;
            let _ = @write(Count(x));
            let y = x + 2;
            let _ = @write(Count(y));
            x + y
        };
        let Writer(v, Count(w)) = w;
        assert_eq!(v, w);
    }
}
