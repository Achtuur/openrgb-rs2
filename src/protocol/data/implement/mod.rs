//! Implements the readable and writable traits for core/std types.

mod array;
mod flags;
mod json;
mod primitive;
mod slice;
mod string;
mod tuple;
mod vec;

pub(crate) use json::*;
pub(crate) use slice::*;
pub(crate) use string::*;
