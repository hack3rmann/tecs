#![feature(vec_into_raw_parts)]
#![allow(dead_code, clippy::missing_safety_doc)]

mod simple;

use std::{alloc::Layout, any::TypeId, collections::HashMap, slice};

type Entity = u32;

pub struct TypeInfo {
    pub id: TypeId,
    pub layout: Layout,
    pub drop: unsafe fn(*mut u8),
}

impl TypeInfo {
    pub fn of<T: 'static>() -> Self {
        unsafe fn drop<T>(ptr: *mut u8) {
            ptr.cast::<T>().drop_in_place();
        }

        Self {
            id: TypeId::of::<T>(),
            layout: Layout::new::<T>(),
            drop: drop::<T>,
        }
    }
}

pub struct Archetype {
    pub(crate) component_types: Box<[TypeInfo]>,
    pub(crate) index: HashMap<TypeId, usize>,
    pub(crate) components: Box<[*mut u8]>,
    pub(crate) len: usize,
    pub(crate) capacity: usize,
    pub(crate) entities: Vec<Entity>,
}

impl Archetype {
    pub fn new<C: Component>() -> Self {
        let (ptr, len, cap) = Vec::<C>::new().into_raw_parts();

        Self {
            component_types: Box::new([TypeInfo::of::<C>()]),
            index: HashMap::from([(TypeId::of::<C>(), 0)]),
            components: Box::new([ptr.cast()]),
            len,
            capacity: cap,
            entities: vec![],
        }
    }

    pub fn add_component<C: Component>(&mut self, value: C) {
        assert_eq!(self.component_types.len(), 1);

        let index = self.index[&TypeId::of::<C>()];

        let mut vec = unsafe {
            Vec::<C>::from_raw_parts(self.components[index].cast(), self.len, self.capacity)
        };
        vec.push(value);

        let (ptr, len, capacity) = vec.into_raw_parts();

        self.components[index] = ptr.cast();
        self.len = len;
        self.capacity = capacity;
    }
}

impl Drop for Archetype {
    fn drop(&mut self) {
        for (i, &components_ptr) in self.components.iter().enumerate() {
            let size = self.component_types[i].layout.size();
            let drop = self.component_types[i].drop;

            for j in 0..self.len {
                unsafe {
                    drop(components_ptr.add(size * j));
                }
            }
        }
    }
}

pub struct Location {
    pub entity_index: u32,
    pub archetype_index: u32,
}

#[derive(Default)]
pub struct World {
    pub(crate) archetypes: Vec<Archetype>,
    pub(crate) locations: Vec<Location>,
    pub(crate) index: HashMap<Box<[TypeId]>, usize>,
}

pub trait Component: Sized + 'static {}

impl World {
    pub fn spawn<C: Component>(&mut self, component: C) -> Entity {
        let entity = self.locations.len() as Entity;

        let archetype_index = match self.index.get(&[TypeId::of::<C>()][..]) {
            Some(&index) => index,
            None => {
                let archetype = Archetype::new::<C>();
                let index = self.archetypes.len();

                self.index.insert(Box::new([TypeId::of::<C>()]), index);
                self.archetypes.push(archetype);

                index
            }
        };

        let entity_index = self.archetypes[archetype_index].len;
        let location = Location {
            entity_index: entity_index as u32,
            archetype_index: archetype_index as u32,
        };

        self.locations.push(location);
        self.archetypes[archetype_index].add_component(component);
        self.archetypes[archetype_index].entities.push(entity);

        entity
    }

    pub fn query_mut<C: Component>(&mut self) -> impl Iterator<Item = (Entity, &mut C)> {
        // TODO: fetch all archetypes that contains C component
        let archetype_index = self.index[&[TypeId::of::<C>()][..]];
        let components_index = self.archetypes[archetype_index].index[&TypeId::of::<C>()];
        let components_ptr = self.archetypes[archetype_index].components[components_index];
        let components_len = self.archetypes[archetype_index].len;
        let components =
            unsafe { slice::from_raw_parts_mut(components_ptr.cast(), components_len) };

        self.archetypes[archetype_index]
            .entities
            .iter()
            .copied()
            .zip(components)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_component_archetype() {
        #[derive(Debug, PartialEq)]
        struct Name(String);
        impl Component for Name {}

        #[derive(Debug, PartialEq)]
        struct Speed(f32);
        impl Component for Speed {}

        let mut world = World::default();

        let entities = [
            world.spawn(Name(String::from("First"))),
            world.spawn(Name(String::from("Second"))),
            world.spawn(Speed(42.0)),
            world.spawn(Name(String::from("Third"))),
            world.spawn(Speed(69.0)),
        ];

        assert_eq!(
            world.query_mut::<Name>().collect::<Vec<_>>(),
            [
                (entities[0], &mut Name(String::from("First"))),
                (entities[1], &mut Name(String::from("Second"))),
                (entities[3], &mut Name(String::from("Third"))),
            ],
        );

        assert_eq!(
            world.query_mut::<Speed>().collect::<Vec<_>>(),
            [
                (entities[2], &mut Speed(42.0)),
                (entities[4], &mut Speed(69.0)),
            ],
        )
    }
}
