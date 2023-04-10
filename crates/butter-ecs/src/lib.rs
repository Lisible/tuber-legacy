#![warn(clippy::pedantic)]
use commands::CommandQueue;

use crate::bitset::Bitset;
use std::{
    alloc::Layout,
    any::{Any, TypeId},
    collections::HashMap,
    ptr::NonNull,
};

use self::system::System;

mod bitset;
pub mod commands;
pub mod query;
pub mod system;

pub struct Ecs {
    next_index: usize,
    deleted_entities_indices: Vec<EntityIndex>,
    component_stores: HashMap<TypeId, ComponentStore>,
    resources: HashMap<TypeId, Box<dyn Any>>,
}

impl Ecs {
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_index: 0,
            deleted_entities_indices: vec![],
            component_stores: HashMap::new(),
            resources: HashMap::new(),
        }
    }

    pub fn insert_resource<R>(&mut self, resource: R)
    where
        R: 'static,
    {
        self.resources.insert(TypeId::of::<R>(), Box::new(resource));
    }

    #[must_use]
    pub fn resource<R>(&self) -> Option<&R>
    where
        R: 'static,
    {
        self.resources
            .get(&TypeId::of::<R>())
            .map(|r| r.downcast_ref())?
    }

    pub fn resource_mut<R>(&mut self) -> Option<&mut R>
    where
        R: 'static,
    {
        self.resources
            .get_mut(&TypeId::of::<R>())
            .map(|r| r.downcast_mut())?
    }

    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.next_index - self.deleted_entities_indices.len()
    }

    pub fn insert<ED>(&mut self, entity_definition: ED) -> EntityIndex
    where
        ED: EntityDefinition,
    {
        let entity_index = self.allocate_index();
        entity_definition.store_component(self, entity_index.index);
        entity_index
    }

    pub fn delete(&mut self, entity_index: EntityIndex) {
        for store in self.component_stores.values_mut() {
            store.remove(entity_index.index);
        }

        self.deleted_entities_indices.push(entity_index);
    }

    pub fn run_systems(&mut self, systems: &mut [Box<dyn System>]) {
        let mut global_command_queue = CommandQueue::new();
        for system in systems.iter_mut() {
            self.run_system(system, &mut global_command_queue);
        }

        self.execute_command_queue(&mut global_command_queue);
    }

    pub fn run_single_system<S>(&mut self, system: &mut S)
    where
        S: System,
    {
        system.run(self);
        self.execute_command_queue(system.command_queue());
    }

    fn run_system<S>(&mut self, system: &mut S, global_command_queue: &mut CommandQueue)
    where
        S: System,
    {
        system.run(self);
        global_command_queue.extend(system.command_queue().drain());
    }

    fn execute_command_queue(&mut self, command_queue: &mut CommandQueue) {
        for command in command_queue.drain() {
            command.execute(self);
        }
    }

    #[must_use]
    pub fn component<C: 'static>(&self, entity_index: EntityIndex) -> Option<&C> {
        self.component_at_index(entity_index.index)
    }

    #[must_use]
    pub fn component_mut<C: 'static>(&self, entity_index: EntityIndex) -> Option<&mut C> {
        self.component_mut_at_index(entity_index.index)
    }

    fn component_at_index<C: 'static>(&self, index: usize) -> Option<&C> {
        self.component_stores
            .get(&TypeId::of::<C>())?
            .get::<C>(index)
    }

    fn component_mut_at_index<C: 'static>(&self, index: usize) -> Option<&mut C> {
        self.component_stores
            .get(&TypeId::of::<C>())?
            .get_mut::<C>(index)
    }

    #[must_use]
    pub fn query<'a, Q>(&'a self) -> query::Iter<'a, Q>
    where
        Q: query::Description<'a>,
    {
        query::Iter::new(self)
    }

    fn allocate_index(&mut self) -> EntityIndex {
        if let Some(reusable_index) = self.deleted_entities_indices.pop() {
            EntityIndex {
                index: reusable_index.index,
                generation: reusable_index.generation + 1,
            }
        } else {
            let next_index = self.next_index;
            let index = EntityIndex {
                index: next_index,
                generation: 0,
            };
            self.next_index += 1;
            index
        }
    }

    fn store_component<C>(&mut self, index: usize, component: C)
    where
        C: 'static,
    {
        let component_store = self
            .component_stores
            .entry(TypeId::of::<C>())
            .or_insert_with(|| {
                ComponentStore::new(std::alloc::Layout::new::<C>(), drop_component_fn::<C>)
            });

        component_store.store(index, component);
    }
}

impl Default for Ecs {
    fn default() -> Self {
        Self::new()
    }
}

pub trait EntityDefinition {
    fn store_component(self, ecs: &mut Ecs, index: usize);
}

macro_rules! impl_entity_definition_for_tuple {
    ($($t:tt: $i:tt,)*) => {
        impl<$($t: 'static,)*> EntityDefinition for ($($t,)*) {
            fn store_component(self, ecs: &mut Ecs, index: usize) {
                $(ecs.store_component(index, self.$i);)*
            }
        }
    }
}

impl_entity_definition_for_tuple!(A: 0,);
impl_entity_definition_for_tuple!(A: 0, B: 1,);
impl_entity_definition_for_tuple!(A: 0, B: 1, C: 2,);
impl_entity_definition_for_tuple!(A: 0, B: 1, C: 2, D: 3,);
impl_entity_definition_for_tuple!(A: 0, B: 1, C: 2, D: 3, E: 4,);
impl_entity_definition_for_tuple!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5,);
impl_entity_definition_for_tuple!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6,);
impl_entity_definition_for_tuple!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7,);
impl_entity_definition_for_tuple!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8,);
impl_entity_definition_for_tuple!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9,);
impl_entity_definition_for_tuple!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10,);
impl_entity_definition_for_tuple!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11,);
impl_entity_definition_for_tuple!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12,);
impl_entity_definition_for_tuple!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13,);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityIndex {
    index: usize,
    generation: usize,
}

const BITSET_BIT_COUNT: usize = 65536;
const fn bitset_word_count<T>() -> usize {
    BITSET_BIT_COUNT / (std::mem::size_of::<T>() * 8)
}

struct ComponentStore {
    data: NonNull<u8>,
    layout: Layout,
    len: usize,
    reserved_len: usize,
    drop: unsafe fn(*mut u8),
    entities_bitset: [u64; bitset_word_count::<u64>()],
}

impl ComponentStore {
    pub fn new(component_layout: Layout, drop: unsafe fn(*mut u8)) -> Self {
        let reserved_len = if component_layout.size() == 0 {
            usize::MAX
        } else {
            0
        };

        Self {
            data: NonNull::dangling(),
            layout: component_layout,
            len: 0,
            reserved_len,
            drop,
            entities_bitset: [0u64; bitset_word_count::<u64>()],
        }
    }

    pub fn store<C>(&mut self, index: usize, mut component: C) {
        assert!(index < BITSET_BIT_COUNT, "ComponentStore is full");
        self.entities_bitset.set_bit(index);
        self.resize(index + 1);

        if self.layout.size() > 0 {
            // SAFETY:
            // The chunked of data has just been resized to ensure it can store the component
            unsafe { self.write(index, std::ptr::addr_of_mut!(component).cast()) };
        }
    }

    pub fn get<C>(&self, index: usize) -> Option<&C> {
        if index >= self.len {
            return None;
        }

        if !self.entities_bitset.bit(index) {
            return None;
        }

        unsafe { Some(&*self.ptr_at(index).cast::<C>()) }
    }

    pub fn get_mut<C>(&self, index: usize) -> Option<&mut C> {
        if index >= self.len {
            return None;
        }

        if !self.entities_bitset.bit(index) {
            return None;
        }

        unsafe { Some(&mut *self.ptr_at(index).cast::<C>()) }
    }

    pub fn ptr(&self) -> *mut u8 {
        self.data.as_ptr()
    }

    /// # Safety
    /// The caller must ensures that index is < self.len
    pub unsafe fn ptr_at(&self, index: usize) -> *mut u8 {
        assert!(index < self.len);
        self.ptr().add(index * self.layout.size())
    }

    pub fn remove(&mut self, index: usize) {
        if !self.entities_bitset.bit(index) || index >= self.len {
            return;
        }

        self.entities_bitset.unset_bit(index);

        // SAFETY:
        // index is inside the bounds of the allocated memory
        unsafe {
            let ptr = self.ptr_at(index);
            (self.drop)(ptr);
        }
        self.len -= 1;
    }

    pub fn clear(&mut self) {
        let len = self.len;
        for i in 0..len {
            self.remove(i);
        }
    }

    fn resize(&mut self, len: usize) {
        self.reserve_exact(len);
        self.len = len;
    }

    fn reserve_exact(&mut self, len: usize) {
        if self.reserved_len >= len {
            return;
        }

        let new_reserved_len = len;
        let new_layout = array_layout(self.layout, new_reserved_len);

        // SAFETY:
        // - The layout is guaranteed to have a non-zero size because we don't reserve
        //   when using ZST
        let new_data = unsafe {
            if self.reserved_len == 0 {
                std::alloc::alloc(new_layout)
            } else {
                std::alloc::realloc(
                    self.data.as_ptr(),
                    array_layout(self.layout, self.reserved_len),
                    new_layout.size(),
                )
            }
        };

        self.reserved_len = new_reserved_len;
        self.data = NonNull::new(new_data).expect("ComponentStore allocation failed");
    }

    /// # Safety
    /// - index must be in the bounds of the allocated chunk of data
    unsafe fn write(&mut self, index: usize, data_ptr: *mut u8) {
        let dst_ptr = self.ptr_at(index);
        std::ptr::copy_nonoverlapping(data_ptr, dst_ptr, self.layout.size());
    }
}

impl Drop for ComponentStore {
    fn drop(&mut self) {
        if self.layout.size() == 0 {
            return;
        }

        self.clear();
        let layout = array_layout(self.layout, self.reserved_len);

        // SAFETY:
        // - self.data has been allocated with the same allocator
        // - the given layout is the same that the one that's been
        //   used to allocate the chunk of memory
        unsafe {
            std::alloc::dealloc(self.data.as_ptr(), layout);
        }
    }
}

unsafe fn drop_component_fn<T>(ptr: *mut u8) {
    ptr.cast::<T>().drop_in_place();
}

// TODO: Remove when std::alloc::Layout::array stabilizes
const fn array_layout(layout: Layout, len: usize) -> Layout {
    let array_size = layout.size() * len;
    assert!(layout.size() != 0 && len <= max_size_for_align(layout.align()) / layout.size());

    // SAFETY: layout being a valid layout
    // - layout.align() is non-zero
    // - layout.align() is a power of two
    // - We checked that when rounded up to the neared multiple of align, array_size doesn't overflow isize
    unsafe { Layout::from_size_align_unchecked(array_size, layout.align()) }
}

const fn max_size_for_align(align: usize) -> usize {
    isize::MAX as usize - (align - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ecs_new() {
        let ecs = Ecs::new();
        assert_eq!(ecs.entity_count(), 0);
    }

    #[derive(Debug, Eq, PartialEq)]
    struct Player;
    #[derive(Debug, Eq, PartialEq)]
    struct Enemy;
    #[derive(Debug, Eq, PartialEq)]
    struct Health(i16);
    #[derive(Debug, Eq, PartialEq)]
    struct Level(u16);

    #[test]
    fn ecs_insert() {
        let mut ecs = Ecs::new();
        let entity_index = ecs.insert((Player, Health(10)));
        assert_eq!(entity_index.index, 0);
        assert_eq!(entity_index.generation, 0);

        let entity_index = ecs.insert((Enemy, Health(5)));
        assert_eq!(entity_index.index, 1);
        assert_eq!(entity_index.generation, 0);

        assert_eq!(ecs.entity_count(), 2);
    }

    #[test]
    fn ecs_remove() {
        let mut ecs = Ecs::new();

        let player = ecs.insert((Player, Health(10)));
        assert_eq!(ecs.entity_count(), 1);

        let enemy = ecs.insert((Enemy, Health(5)));
        assert_eq!(ecs.entity_count(), 2);

        ecs.delete(enemy);
        assert_eq!(ecs.entity_count(), 1);

        ecs.delete(player);
        assert_eq!(ecs.entity_count(), 0);
    }

    #[test]
    fn ecs_reuse_index() {
        let mut ecs = Ecs::new();

        let player = ecs.insert((Player, Health(10)));
        assert_eq!(player.index, 0);
        assert_eq!(player.generation, 0);

        ecs.delete(player);

        let player = ecs.insert((Player, Health(5)));
        assert_eq!(player.index, 0);
        assert_eq!(player.generation, 1);
    }

    #[test]
    fn ecs_component() {
        let mut ecs = Ecs::new();
        let player = ecs.insert((Player, Health(10)));
        let player_component = ecs.component::<Player>(player).unwrap();
        assert_eq!(player_component, &Player);
        let health_component = ecs.component::<Health>(player).unwrap();
        assert_eq!(health_component, &Health(10));

        assert_eq!(ecs.component::<Enemy>(player), None);
    }

    #[test]
    fn ecs_component_mut() {
        let mut ecs = Ecs::new();
        let player = ecs.insert((Player, Health(10)));
        let health_component = ecs.component_mut::<Health>(player).unwrap();
        health_component.0 = 5;
        let health_component = ecs.component::<Health>(player).unwrap();
        assert_eq!(health_component, &Health(5));
    }

    #[test]
    fn ecs_query() {
        let mut ecs = Ecs::new();
        let _player = ecs.insert((Player, Health(10)));
        let _enemy = ecs.insert((Enemy, Health(5)));
        let mut health_iter = ecs.query::<&Health>();
        assert_eq!(health_iter.next(), Some(&Health(10)));
        assert_eq!(health_iter.next(), Some(&Health(5)));
        assert_eq!(health_iter.next(), None);
    }

    #[test]
    fn ecs_query_multiple() {
        let mut ecs = Ecs::new();
        let _player = ecs.insert((Player, Level(1), Health(10)));
        let _enemy = ecs.insert((Enemy, Health(5)));
        let mut query_iter = ecs.query::<(&Player, &Level, &mut Health)>();
        assert_eq!(
            query_iter.next(),
            Some((&Player, &Level(1), &mut Health(10)))
        );
        assert_eq!(query_iter.next(), None);
    }

    #[test]
    fn ecs_query_mut() {
        let mut ecs = Ecs::new();
        let _player = ecs.insert((Player, Health(10)));
        let _enemy = ecs.insert((Enemy, Health(5)));
        for health in ecs.query::<&mut Health>() {
            health.0 = 0;
        }

        let mut health_iter = ecs.query::<&Health>();
        assert_eq!(health_iter.next(), Some(&Health(0)));
        assert_eq!(health_iter.next(), Some(&Health(0)));
        assert_eq!(health_iter.next(), None);
    }

    #[test]
    fn ecs_insert_resource() {
        #[derive(PartialEq, Eq, Debug)]
        struct DeltaTime(u32);
        let mut ecs = Ecs::new();
        ecs.insert_resource(DeltaTime(25));

        {
            let fetched_delta_time = ecs.resource::<DeltaTime>().unwrap();
            assert_eq!(fetched_delta_time, &DeltaTime(25));
        }

        {
            let mutably_fetched_deltatime = ecs.resource_mut::<DeltaTime>().unwrap();
            assert_eq!(mutably_fetched_deltatime, &mut DeltaTime(25));
            mutably_fetched_deltatime.0 = 10;
            assert_eq!(mutably_fetched_deltatime, &mut DeltaTime(10));
        }

        {
            let fetched_delta_time = ecs.resource::<DeltaTime>().unwrap();
            assert_eq!(fetched_delta_time, &DeltaTime(10));
        }
    }
}
