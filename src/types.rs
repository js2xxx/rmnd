use core::marker::Tuple;

#[derive(Debug, Clone, Copy)]
pub struct Identity;

impl<T> FnOnce<(T,)> for Identity {
    type Output = T;

    extern "rust-call" fn call_once(self, (t,): (T,)) -> T {
        t
    }
}

impl<T> FnMut<(T,)> for Identity {
    extern "rust-call" fn call_mut(&mut self, (t,): (T,)) -> T {
        t
    }
}

impl<T> Fn<(T,)> for Identity {
    extern "rust-call" fn call(&self, (t,): (T,)) -> T {
        t
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Value<T>(pub T);

impl<A: Tuple, T> FnOnce<A> for Value<T> {
    type Output = T;

    extern "rust-call" fn call_once(self, _: A) -> T {
        self.0
    }
}

impl<A: Tuple, T: Clone> FnMut<A> for Value<T> {
    extern "rust-call" fn call_mut(&mut self, _: A) -> T {
        self.0.clone()
    }
}

impl<A: Tuple, T: Clone> Fn<A> for Value<T> {
    extern "rust-call" fn call(&self, _: A) -> T {
        self.0.clone()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Duplicate;

impl<T: Clone> FnOnce<(T,)> for Duplicate {
    type Output = (T, T);

    extern "rust-call" fn call_once(self, (t,): (T,)) -> (T, T) {
        (t.clone(), t)
    }
}

impl<T: Clone> FnMut<(T,)> for Duplicate {
    extern "rust-call" fn call_mut(&mut self, (t,): (T,)) -> (T, T) {
        (t.clone(), t)
    }
}

impl<T: Clone> Fn<(T,)> for Duplicate {
    extern "rust-call" fn call(&self, (t,): (T,)) -> (T, T) {
        (t.clone(), t)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Append<T>(pub T);

impl<U, T> FnOnce<(U,)> for Append<T> {
    type Output = (T, U);

    extern "rust-call" fn call_once(self, (u,): (U,)) -> (T, U) {
        (self.0, u)
    }
}

impl<U, T: Clone> FnMut<(U,)> for Append<T> {
    extern "rust-call" fn call_mut(&mut self, (u,): (U,)) -> (T, U) {
        (self.0.clone(), u)
    }
}

impl<U, T: Clone> Fn<(U,)> for Append<T> {
    extern "rust-call" fn call(&self, (u,): (U,)) -> (T, U) {
        (self.0.clone(), u)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Replace<T>(pub T);

impl<T> FnOnce<(T,)> for Replace<T> {
    type Output = (T, T);

    extern "rust-call" fn call_once(self, (t,): (T,)) -> (T, T) {
        (self.0, t)
    }
}

impl<T: Clone> FnMut<(T,)> for Replace<T> {
    extern "rust-call" fn call_mut(&mut self, (t,): (T,)) -> (T, T) {
        (self.0.clone(), t)
    }
}

impl<T: Clone> Fn<(T,)> for Replace<T> {
    extern "rust-call" fn call(&self, (t,): (T,)) -> (T, T) {
        (self.0.clone(), t)
    }
}
