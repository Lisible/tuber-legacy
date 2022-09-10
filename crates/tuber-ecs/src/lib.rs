#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]

#[macro_use]
extern crate assert_float_eq;

mod bitset;
pub mod ecs;
pub mod query;
pub mod system;

/// The index of an entity
pub type EntityIndex = usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd)]
pub struct Parent(pub EntityIndex);
