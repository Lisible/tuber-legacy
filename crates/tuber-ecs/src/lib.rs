mod bitset;
pub mod ecs;
pub mod query;
pub mod system;

/// The index of an entity
pub type EntityIndex = usize;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Parent(pub EntityIndex);
