use std::any::TypeId;
use std::collections::HashSet;
use std::marker::PhantomData;

use accessors::Accessor;

use crate::bitset::BitSet;
use crate::ecs::Components;
use crate::query::ComponentTypeId::{OptionalComponentTypeId, RequiredComponentTypeId};
use crate::EntityIndex;

pub trait Query<'a> {
    type ResultType: 'a;

    fn fetch(index: EntityIndex, components: &'a Components) -> Option<Self::ResultType>;
    fn matching_ids(entity_count: usize, components: &'a Components) -> HashSet<EntityIndex>;
    fn type_ids() -> Vec<ComponentTypeId>;
}

macro_rules! impl_query_tuples {
    ($th:tt, $($t:tt,)*) => {
        impl<'a, $th, $($t,)*> Query<'a> for ($th, $($t,)*)
        where
            $th: Accessor<'a>,
            $($t: Accessor<'a>,)*
        {
            type ResultType = (EntityIndex, ($th::RefType, $($t::RefType,)*));

            fn fetch(index: EntityIndex, components: &'a Components) -> Option<Self::ResultType> {
                Some((index, ($th::fetch(index, components)?, $($t::fetch(index, components)?,)*)))
            }

            #[allow(unused_mut)]
            fn matching_ids(entity_count: usize, components: &'a Components) -> HashSet<EntityIndex> {
                let mut result = $th::matching_ids(entity_count, components);
                $(result = result.intersection(&$t::matching_ids(entity_count, components)).cloned().collect();)*
                result
            }

            fn type_ids() -> Vec<ComponentTypeId> {
                vec![$th::type_id(), $($t::type_id(),)*]
            }
        }
    }
}

impl_query_tuples!(A,);
impl_query_tuples!(A, B,);
impl_query_tuples!(A, B, C,);
impl_query_tuples!(A, B, C, D,);
impl_query_tuples!(A, B, C, D, E,);
impl_query_tuples!(A, B, C, D, E, F,);
impl_query_tuples!(A, B, C, D, E, F, G,);
impl_query_tuples!(A, B, C, D, E, F, G, H,);

pub struct QueryIteratorByIds<'a, Q> {
    inner_iterator: QueryIterator<'a, Q>,
    ids: HashSet<usize>,
}

impl<'a, 'b, Q: Query<'b>> QueryIteratorByIds<'a, Q> {
    #[must_use]
    pub fn new(entity_count: usize, components: &'a Components, ids: &HashSet<usize>) -> Self {
        Self {
            inner_iterator: QueryIterator::new(entity_count, components),
            ids: ids.iter().copied().collect(),
        }
    }
}

impl<'a, Q> Iterator for QueryIteratorByIds<'a, Q>
where
    Q: Query<'a>,
{
    type Item = Q::ResultType;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next = self.inner_iterator.next();
        while !self.ids.contains(&self.inner_iterator.index) && next.is_some() {
            next = self.inner_iterator.next();
        }

        next
    }
}

pub struct QueryIterator<'a, Q> {
    index: EntityIndex,
    components: &'a Components,
    matching_entities: Vec<EntityIndex>,
    marker: PhantomData<&'a Q>,
}

impl<'a, 'b, Q: Query<'b>> QueryIterator<'a, Q> {
    #[must_use]
    pub fn new(entity_count: usize, components: &'a Components) -> Self {
        let mut bitsets = vec![];
        for type_id in Q::type_ids() {
            match type_id {
                RequiredComponentTypeId(type_id) => {
                    if let Some(component_store) = components.get(&type_id) {
                        bitsets.push(component_store.entities_bitset);
                    }
                }
                OptionalComponentTypeId(_) => continue,
            }
        }

        let mut matching_entities = vec![];
        if bitsets.len() == Q::type_ids().iter().filter(|t| t.is_required()).count() {
            'outer: for i in 0..entity_count {
                for bitset in &bitsets {
                    if !bitset.bit(i) {
                        continue 'outer;
                    }
                }

                matching_entities.push(i);
            }
        }

        Self {
            index: 0,
            components,
            matching_entities,
            marker: PhantomData,
        }
    }
}

impl<'a, Q> Iterator for QueryIterator<'a, Q>
where
    Q: Query<'a>,
{
    type Item = Q::ResultType;

    fn next(&mut self) -> Option<Self::Item> {
        self.index = self.matching_entities.pop()?;
        Q::fetch(self.index, self.components)
    }
}

pub mod accessors {
    use std::any::TypeId;
    use std::cell::{Ref, RefMut};
    use std::collections::HashSet;
    use std::marker::PhantomData;

    use crate::bitset::BitSet;
    use crate::ecs::Components;
    use crate::query::ComponentTypeId;
    use crate::query::ComponentTypeId::{OptionalComponentTypeId, RequiredComponentTypeId};
    use crate::EntityIndex;

    pub struct Opt<'a, T: Accessor<'a>>(PhantomData<&'a T>);

    pub trait Accessor<'a> {
        type RawType: 'a;
        type RefType: 'a;

        fn fetch(index: usize, components: &'a Components) -> Option<Self::RefType>;
        fn matching_ids(entity_count: usize, components: &'a Components) -> HashSet<EntityIndex>;
        fn type_id() -> ComponentTypeId;
    }

    impl<'a, T: 'static> Accessor<'a> for &T {
        type RawType = T;
        type RefType = Ref<'a, T>;

        fn fetch(index: usize, components: &'a Components) -> Option<Self::RefType> {
            Some(Ref::map(
                components.get(&TypeId::of::<T>())?.component_data[index]
                    .as_ref()?
                    .borrow(),
                |r| r.downcast_ref().unwrap(),
            ))
        }

        fn matching_ids(entity_count: usize, components: &'a Components) -> HashSet<EntityIndex> {
            matching_ids_for_type::<T>(entity_count, components)
        }

        fn type_id() -> ComponentTypeId {
            RequiredComponentTypeId(TypeId::of::<T>())
        }
    }

    impl<'a, T: 'static> Accessor<'a> for &mut T {
        type RawType = T;
        type RefType = RefMut<'a, T>;

        fn fetch(index: usize, components: &'a Components) -> Option<Self::RefType> {
            Some(RefMut::map(
                components.get(&TypeId::of::<T>())?.component_data[index]
                    .as_ref()?
                    .borrow_mut(),
                |r| r.downcast_mut().unwrap(),
            ))
        }

        fn matching_ids(entity_count: usize, components: &'a Components) -> HashSet<EntityIndex> {
            matching_ids_for_type::<T>(entity_count, components)
        }

        fn type_id() -> ComponentTypeId {
            RequiredComponentTypeId(TypeId::of::<T>())
        }
    }

    fn matching_ids_for_type<T: 'static>(
        entity_count: usize,
        components: &Components,
    ) -> HashSet<EntityIndex> {
        let mut result = HashSet::new();
        if let Some(component_store) = components.get(&TypeId::of::<T>()) {
            for i in 0..entity_count.max(component_store.entities_bitset.bit_count()) {
                if component_store.entities_bitset.bit(i) {
                    result.insert(i);
                }
            }
        }

        result
    }

    impl<'a, T: 'static + Accessor<'a>> Accessor<'a> for Opt<'a, T> {
        type RawType = T::RawType;
        type RefType = Option<T::RefType>;

        fn fetch(index: usize, components: &'a Components) -> Option<Self::RefType> {
            Some(T::fetch(index, components))
        }

        fn matching_ids(entity_count: usize, _components: &'a Components) -> HashSet<EntityIndex> {
            (0..entity_count).collect()
        }

        fn type_id() -> ComponentTypeId {
            if let RequiredComponentTypeId(type_id) = T::type_id() {
                OptionalComponentTypeId(type_id)
            } else {
                panic!("Can't use nested OptionalComponentTypeId")
            }
        }
    }
}

pub enum ComponentTypeId {
    RequiredComponentTypeId(TypeId),
    OptionalComponentTypeId(TypeId),
}

impl ComponentTypeId {
    #[must_use]
    pub fn is_required(&self) -> bool {
        matches!(self, RequiredComponentTypeId(_))
    }
}
