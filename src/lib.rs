#![no_std]
#![feature(fn_traits)]
#![feature(tuple_trait)]
#![feature(unboxed_closures)]

pub mod reader;
pub mod state;
mod types;

#[cfg(test)]
extern crate std;
