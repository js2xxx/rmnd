use core::marker::PhantomData;

use crate::types::{Append, Duplicate, Replace, Value};

pub struct State<T, S, F: FnOnce(S) -> (T, S)>(F, PhantomData<S>);

impl<T, S, F: FnOnce(S) -> (T, S) + Clone> Clone for State<T, S, F> {
    fn clone(&self) -> Self {
        State(self.0.clone(), PhantomData)
    }
}

impl<T, S, F: FnOnce(S) -> (T, S) + Copy> Copy for State<T, S, F> {}

impl<T, S, F: FnOnce(S) -> (T, S)> FnOnce<(S,)> for State<T, S, F> {
    type Output = (T, S);

    extern "rust-call" fn call_once(self, args: (S,)) -> Self::Output {
        (self.0)(args.0)
    }
}

impl<T, S, F: FnMut(S) -> (T, S)> FnMut<(S,)> for State<T, S, F> {
    extern "rust-call" fn call_mut(&mut self, args: (S,)) -> Self::Output {
        (self.0)(args.0)
    }
}

impl<T, S, F: Fn(S) -> (T, S)> Fn<(S,)> for State<T, S, F> {
    extern "rust-call" fn call(&self, args: (S,)) -> Self::Output {
        (self.0)(args.0)
    }
}

impl<T, S, F: FnOnce(S) -> (T, S)> State<T, S, F> {
    pub fn bind<U, G, H>(self, f: H) -> State<U, S, Bind<T, S, F, U, G, H>>
    where
        G: FnOnce(S) -> (U, S),
        H: FnOnce(T) -> State<U, S, G>,
    {
        State(Bind(self.0, f, PhantomData), PhantomData)
    }
}

pub struct Bind<T, S, F, U, G, H>(F, H, PhantomData<S>)
where
    F: FnOnce(S) -> (T, S),
    G: FnOnce(S) -> (U, S),
    H: FnOnce(T) -> State<U, S, G>;

impl<T, S, F, U, G, H> Clone for Bind<T, S, F, U, G, H>
where
    F: FnOnce(S) -> (T, S) + Clone,
    G: FnOnce(S) -> (U, S),
    H: FnOnce(T) -> State<U, S, G> + Clone,
{
    fn clone(&self) -> Self {
        Bind(self.0.clone(), self.1.clone(), PhantomData)
    }
}

impl<T, S, F, U, G, H> Copy for Bind<T, S, F, U, G, H>
where
    F: FnOnce(S) -> (T, S) + Copy,
    G: FnOnce(S) -> (U, S),
    H: FnOnce(T) -> State<U, S, G> + Copy,
{
}

impl<T, S, F, U, G, H> FnOnce<(S,)> for Bind<T, S, F, U, G, H>
where
    F: FnOnce(S) -> (T, S),
    G: FnOnce(S) -> (U, S),
    H: FnOnce(T) -> State<U, S, G>,
{
    type Output = (U, S);

    extern "rust-call" fn call_once(self, (s,): (S,)) -> (U, S) {
        let (value, s) = (self.0)(s);
        ((self.1)(value).0)(s)
    }
}

impl<T, S, F, U, G, H> FnMut<(S,)> for Bind<T, S, F, U, G, H>
where
    F: FnMut(S) -> (T, S),
    G: FnOnce(S) -> (U, S),
    H: FnMut(T) -> State<U, S, G>,
{
    extern "rust-call" fn call_mut(&mut self, (s,): (S,)) -> (U, S) {
        let (value, s) = (self.0)(s);
        ((self.1)(value).0)(s)
    }
}

impl<T, S, F, U, G, H> Fn<(S,)> for Bind<T, S, F, U, G, H>
where
    F: Fn(S) -> (T, S),
    G: FnOnce(S) -> (U, S),
    H: Fn(T) -> State<U, S, G>,
{
    extern "rust-call" fn call(&self, (s,): (S,)) -> (U, S) {
        let (value, s) = (self.0)(s);
        ((self.1)(value).0)(s)
    }
}

pub fn value<T, S>(value: T) -> State<T, S, Append<T>> {
    State(Append(value), PhantomData)
}

pub fn read<S: Clone>() -> State<S, S, Duplicate> {
    State(Duplicate, PhantomData)
}

pub fn write<S>(s: S) -> State<(), S, Value<((), S)>> {
    State(Value(((), s)), PhantomData)
}

pub fn replace<S>(s: S) -> State<S, S, Replace<S>> {
    State(Replace(s), PhantomData)
}

#[macro_export]
macro_rules! state {
    {@ $e:expr} => ($e);
    {$e:expr} => ($crate::state::value($e));
    {let $v:pat = @ $e:expr $(=> $ty:ty)?; $($t:tt)*} => {{
        let closure = move |$v $(:$ty)?| state!($($t)*);
        $e .bind(closure)
    }};
    {let $v:pat = $e:expr $(=> $ty:ty)?; $($t:tt)*} => {{
        let closure = move |$v $(:$ty)?| state!($($t)*);
        $crate::state::value($e).bind(closure)
    }};
}

#[cfg(test)]
mod tests {
    use super::{read, write};

    #[test]
    fn basic() {
        let m = state! {
            let x = @read();
            let y = x + 1;
            let _ = @write(y);
            x + y
        };
        assert_eq!(m(1), (3, 2));
        assert_eq!(m(2), (5, 3));
        assert_eq!(m(3), (7, 4));
    }
}
