//! The ecs module defines the Ecs struct which is the main entry point of tuber-ecs

use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::{HashMap, HashSet};

use crate::bitset::BitSet;
use crate::query::{ComponentTypeId, Query, QueryIterator, QueryIteratorByIds};
use crate::EntityIndex;

pub type Components = HashMap<TypeId, ComponentStore>;
pub type Resources = HashMap<TypeId, RefCell<Box<dyn Any>>>;

type EntitiesBitsetType = [u64; 1024];

pub struct ComponentStore {
    pub(crate) component_data: Vec<Option<RefCell<Box<dyn Any>>>>,
    pub(crate) entities_bitset: EntitiesBitsetType,
}

impl ComponentStore {
    #[must_use]
    pub fn with_size(size: usize) -> Self {
        let mut component_data = vec![None];
        for _ in 0..size {
            component_data.push(None);
        }

        Self {
            component_data,
            entities_bitset: [0u64; 1024],
        }
    }

    pub fn remove_from_entity(&mut self, entity_index: EntityIndex) {
        self.entities_bitset.unset_bit(entity_index);
        self.component_data[entity_index] = None;
    }

    pub fn add_to_entity<C: 'static>(&mut self, component: C, entity_index: EntityIndex) {
        self.entities_bitset.set_bit(entity_index);
        self.component_data[entity_index] = Some(RefCell::new(Box::new(component)));
    }
}

impl Default for ComponentStore {
    fn default() -> Self {
        Self {
            component_data: vec![None],
            entities_bitset: [0u64; 1024],
        }
    }
}

/// The Ecs itself, stores entities and runs systems
#[derive(Default)]
pub struct Ecs {
    components: Components,
    shared_resources: Resources,
    next_index: EntityIndex,
}

impl Ecs {
    pub fn insert_shared_resource<T: 'static>(&mut self, resource: T) {
        self.shared_resources
            .insert(TypeId::of::<T>(), RefCell::new(Box::new(resource)));
    }

    #[must_use]
    pub fn shared_resource<T: 'static>(&self) -> Option<Ref<T>> {
        Some(Ref::map(
            self.shared_resources.get(&TypeId::of::<T>())?.borrow(),
            |r| r.downcast_ref().unwrap(),
        ))
    }

    #[must_use]
    pub fn shared_resource_mut<T: 'static>(&self) -> Option<RefMut<T>> {
        Some(RefMut::map(
            self.shared_resources
                .get(&TypeId::of::<T>())
                .as_ref()?
                .borrow_mut(),
            |r| r.downcast_mut().unwrap(),
        ))
    }

    /// Inserts an entity into the Ecs.
    ///
    /// This method takes an [`EntityDefinition`] describing the entity.
    ///
    /// It returns the [`EntityIndex`] of the inserted entity.
    pub fn insert<ED: EntityDefinition>(&mut self, entity_definition: ED) -> EntityIndex {
        let index = self.next_index;
        entity_definition.store_components(&mut self.components, index);
        self.next_index += 1;
        index
    }

    pub fn delete_by_query<Q: for<'a> Query<'a>>(&mut self) {
        let to_delete = Q::matching_ids(self.entity_count(), &self.components);
        self.delete_by_ids(to_delete.iter().copied().collect::<Vec<_>>().as_slice());
    }

    pub fn delete_by_ids(&mut self, to_delete: &[usize]) {
        for &entity_index in to_delete {
            for component in self.components.values_mut() {
                component.entities_bitset.unset_bit(entity_index);
                component.component_data[entity_index] = None;
            }
        }
    }

    pub fn remove_component<C: 'static>(&mut self, entity_index: EntityIndex) {
        if let Some(components) = self.components.get_mut(&TypeId::of::<C>()) {
            components.remove_from_entity(entity_index);
        }
    }

    pub fn add_component<C: 'static>(&mut self, component: C, entity_index: EntityIndex) {
        if let Some(components) = self.components.get_mut(&TypeId::of::<C>()) {
            components.add_to_entity(component, entity_index);
        }
    }

    #[must_use]
    pub fn query<'a, Q: Query<'a>>(&self) -> QueryIterator<Q> {
        QueryIterator::new(self.entity_count(), &self.components)
    }

    #[must_use]
    pub fn query_by_ids<'a, Q: Query<'a>>(&self, ids: &HashSet<usize>) -> QueryIteratorByIds<Q> {
        QueryIteratorByIds::new(self.entity_count(), &self.components, ids)
    }

    #[must_use]
    pub fn query_one<'a, Q: Query<'a>>(&'a self) -> Option<Q::ResultType> {
        let index = {
            let type_ids = Q::type_ids();
            let type_ids: Vec<_> = type_ids
                .iter()
                .filter_map(|type_id| {
                    if let ComponentTypeId::RequiredComponentTypeId(type_id) = type_id {
                        Some(type_id)
                    } else {
                        None
                    }
                })
                .collect();
            let bitsets: Vec<&EntitiesBitsetType> = type_ids
                .iter()
                .filter_map(|type_id| Some(&self.components.get(type_id)?.entities_bitset))
                .collect();
            if bitsets.len() != type_ids.len() {
                return None;
            }

            let mut index = None;
            for i in 0..self.entity_count() {
                if bitsets.iter().all(|bitset| bitset.bit(i)) {
                    index = Some(i);
                    break;
                }
            }
            index?
        };

        Q::fetch(index, &self.components)
    }

    #[must_use]
    pub fn query_one_by_id<'a, Q: Query<'a>>(&'a self, id: EntityIndex) -> Option<Q::ResultType> {
        Q::fetch(id, &self.components)
    }

    /// Returns the entity count of the Ecs.
    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.next_index
    }
}

/// A type that can be used to define an entity
pub trait EntityDefinition {
    fn store_components(self, components: &mut Components, index: usize);
}

macro_rules! impl_entity_definition_tuples {
    ($($t:tt => $i:tt,)*) => {
        impl<$($t: 'static,)*> EntityDefinition for ($($t,)*) {
            fn store_components(self, components: &mut Components, index: usize) {
                use crate::bitset::BitSet;

                for component_storage in components.values_mut() {
                    component_storage.component_data.push(None);
                }

                $(
                    let component_storage = components.entry(TypeId::of::<$t>()).or_insert(ComponentStore::with_size(index));
                    *component_storage.component_data.last_mut().unwrap() = (Some(RefCell::new(Box::new(self.$i))));
                    component_storage.entities_bitset.set_bit(index);
                )*
            }
        }
    }
}

impl_entity_definition_tuples!(A => 0,);
impl_entity_definition_tuples!(A => 0, B => 1,);
impl_entity_definition_tuples!(A => 0, B => 1, C => 2,);
impl_entity_definition_tuples!(A => 0, B => 1, C => 2, D => 3,);
impl_entity_definition_tuples!(A => 0, B => 1, C => 2, D => 3, E => 4,);
impl_entity_definition_tuples!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5,);
impl_entity_definition_tuples!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6,);
impl_entity_definition_tuples!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7,);

#[cfg(test)]
mod tests {
    use crate::query::accessors::Opt;

    use super::*;

    #[derive(Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Debug)]
    struct Velocity {
        x: f32,
        y: f32,
    }

    #[test]
    pub fn ecs_new() {
        let ecs = Ecs::default();
        assert_eq!(ecs.entity_count(), 0usize);
    }

    #[test]
    pub fn ecs_insert() {
        let mut ecs = Ecs::default();
        ecs.insert((Position { x: 0.0, y: 1.0 }, Velocity { x: 2.0, y: 3.0 }));
        assert_eq!(ecs.entity_count(), 1usize);
        ecs.insert((Position { x: 4.0, y: 5.0 }, Velocity { x: 6.0, y: 7.0 }));
        assert_eq!(ecs.entity_count(), 2usize);
    }

    #[test]
    pub fn ecs_query() {
        let mut ecs = Ecs::default();
        ecs.insert((Position { x: 12.0, y: 1.0 }, Velocity { x: 2.0, y: 3.0 }));
        ecs.insert((Position { x: 4.0, y: 5.0 }, Velocity { x: 6.0, y: 7.0 }));
        ecs.insert((Position { x: 4.0, y: 5.0 },));

        for (_, (mut velocity,)) in ecs.query::<(&mut Velocity,)>() {
            velocity.x = 0.0;
        }

        for (_, (velocity,)) in ecs.query::<(&Velocity,)>() {
            assert_float_absolute_eq!(velocity.x, 0.0, 0.01);
        }

        assert_eq!(ecs.query::<(&Position,)>().count(), 3);
        assert_eq!(ecs.query::<(&Velocity,)>().count(), 2);
    }

    #[test]
    pub fn ecs_query_one() {
        let mut ecs = Ecs::default();
        ecs.insert((Position { x: 12.0, y: 1.0 }, Velocity { x: 2.0, y: 3.0 }));

        assert_eq!(ecs.query_one::<(&Position,)>().unwrap().0, 0);
        assert_eq!(
            (*(ecs.query_one::<(&Position,)>().unwrap().1).0),
            Position { x: 12.0, y: 1.0 }
        );
    }

    #[test]
    pub fn ecs_query_optional() {
        let mut ecs = Ecs::default();
        ecs.insert((Position { x: 12.0, y: 1.0 }, Velocity { x: 2.0, y: 3.0 }));
        ecs.insert((Position { x: 15.0, y: 8.0 },));

        assert_eq!(ecs.query::<(&Position, Opt<&Velocity>)>().count(), 2);
    }

    #[test]
    pub fn ecs_query_one_optional() {
        let mut ecs = Ecs::default();
        ecs.insert((Position { x: 12.0, y: 1.0 },));

        let (_, (position, velocity)) = ecs.query_one::<(&Position, Opt<&Velocity>)>().unwrap();

        assert_eq!(*position, Position { x: 12.0, y: 1.0 });
        assert!(velocity.is_none());
    }

    #[test]
    pub fn ecs_query_one_implicit_read_accessor() {
        let mut ecs = Ecs::default();
        ecs.insert((Position { x: 12.0, y: 1.0 },));

        let (_, (position,)) = ecs.query_one::<(&Position,)>().unwrap();

        assert_eq!(*position, Position { x: 12.0, y: 1.0 });
    }
}
