use core::marker::PhantomData;

use crate::types::{Identity, Value};

pub struct Reader<T, R, F: FnOnce(R) -> T>(F, PhantomData<R>);

impl<T, R, F: FnOnce(R) -> T + Clone> Clone for Reader<T, R, F> {
    fn clone(&self) -> Self {
        Reader(self.0.clone(), PhantomData)
    }
}

impl<T, R, F: FnOnce(R) -> T + Copy> Copy for Reader<T, R, F> {}

impl<T, R, F: FnOnce(R) -> T> FnOnce<(R,)> for Reader<T, R, F> {
    type Output = T;

    extern "rust-call" fn call_once(self, args: (R,)) -> Self::Output {
        (self.0)(args.0)
    }
}

impl<T, R, F: FnMut(R) -> T> FnMut<(R,)> for Reader<T, R, F> {
    extern "rust-call" fn call_mut(&mut self, args: (R,)) -> Self::Output {
        (self.0)(args.0)
    }
}

impl<T, R, F: Fn(R) -> T> Fn<(R,)> for Reader<T, R, F> {
    extern "rust-call" fn call(&self, args: (R,)) -> Self::Output {
        (self.0)(args.0)
    }
}

impl<T, R, F: FnOnce(R) -> T> Reader<T, R, F> {
    pub fn bind<U, G, H>(self, h: H) -> Reader<U, R, Bind<T, R, F, U, G, H>>
    where
        R: Clone,
        G: FnOnce(R) -> U,
        H: FnOnce(T) -> Reader<U, R, G>,
    {
        Reader(Bind(self.0, h, PhantomData), PhantomData)
    }
}

pub struct Bind<T, R, F, U, G, H>(F, H, PhantomData<R>)
where
    F: FnOnce(R) -> T,
    G: FnOnce(R) -> U,
    H: FnOnce(T) -> Reader<U, R, G>;

impl<T, R, F, U, G, H> Clone for Bind<T, R, F, U, G, H>
where
    F: FnOnce(R) -> T + Clone,
    G: FnOnce(R) -> U,
    H: FnOnce(T) -> Reader<U, R, G> + Clone,
{
    fn clone(&self) -> Self {
        Bind(self.0.clone(), self.1.clone(), PhantomData)
    }
}

impl<T, R, F, U, G, H> Copy for Bind<T, R, F, U, G, H>
where
    F: FnOnce(R) -> T + Copy,
    G: FnOnce(R) -> U,
    H: FnOnce(T) -> Reader<U, R, G> + Copy,
{
}

impl<T, R, F, U, G, H> FnOnce<(R,)> for Bind<T, R, F, U, G, H>
where
    R: Clone,
    F: FnOnce(R) -> T,
    G: FnOnce(R) -> U,
    H: FnOnce(T) -> Reader<U, R, G>,
{
    type Output = U;

    extern "rust-call" fn call_once(self, (r,): (R,)) -> Self::Output {
        ((self.1)((self.0)(r.clone())).0)(r)
    }
}

impl<T, R, F, U, G, H> FnMut<(R,)> for Bind<T, R, F, U, G, H>
where
    R: Clone,
    F: FnMut(R) -> T,
    G: FnOnce(R) -> U,
    H: FnMut(T) -> Reader<U, R, G>,
{
    extern "rust-call" fn call_mut(&mut self, (r,): (R,)) -> Self::Output {
        ((self.1)((self.0)(r.clone())).0)(r)
    }
}

impl<T, R, F, U, G, H> Fn<(R,)> for Bind<T, R, F, U, G, H>
where
    R: Clone,
    F: Fn(R) -> T,
    G: FnOnce(R) -> U,
    H: Fn(T) -> Reader<U, R, G>,
{
    extern "rust-call" fn call(&self, (r,): (R,)) -> Self::Output {
        ((self.1)((self.0)(r.clone())).0)(r)
    }
}

pub fn read<R>() -> Reader<R, R, Identity> {
    Reader(Identity, PhantomData)
}

pub fn value<T, R>(value: T) -> Reader<T, R, Value<T>> {
    Reader(Value(value), PhantomData)
}

#[macro_export]
macro_rules! reader {
    {@ $e:expr} => ($e);
    {$e:expr} => ($crate::reader::value($e));
    {let $v:pat = @ $e:expr $(=> $ty:ty)?; $($t:tt)*} => {{
        let closure = move |$v $(:$ty)?| reader!($($t)*);
        $e .bind(closure)
    }};
    {let $v:pat = $e:expr $(=> $ty:ty)?; $($t:tt)*} => {{
        let closure = move |$v $(:$ty)?| reader!($($t)*);
        $crate::reader::value($e).bind(closure)
    }};
}

#[cfg(test)]
mod tests {
    use std::string::String;

    use std::string::ToString;

    use quickcheck_macros::quickcheck;

    use super::read;

    #[quickcheck]
    fn basic(input: i16) {
        let dup = reader! {
            let x = @read() => i32;
            let y = @read();
            x + y
        };
        assert_eq!(dup(input as i32), (input as i32) * 2);
        assert_eq!(dup(input as i32), (input as i32) * 2);
    }

    #[test]
    fn nested() {
        let tom = reader! {
            let tom = @read();
            tom + "Tom"
        };
        assert_eq!(tom("A".to_string()), "ATom");
        let jerry = reader! {
            let jerry = @read();
            jerry + "Jerry"
        };
        assert_eq!(jerry("A".to_string()), "AJerry");
        let tom_jerry = reader! {
            let tom = @tom;
            let jerry = @jerry => String;
            tom + " and " + &*jerry
        };
        assert_eq!(tom_jerry("A".to_string()), "ATom and AJerry");
        assert_eq!(tom_jerry("A".to_string()), "ATom and AJerry");
    }
}
