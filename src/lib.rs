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
    pub(crate) index: HashMap<TypeId, usize>,
    pub(crate) component_types: Box<[TypeInfo]>,
    pub(crate) components: Box<[*mut u8]>,
    pub(crate) capacity: usize,
    pub(crate) entities: Vec<Entity>,
}

impl Archetype {
    pub fn new<C: Component>() -> Self {
        Self {
            component_types: Box::new([TypeInfo::of::<C>()]),
            index: HashMap::from([(TypeId::of::<C>(), 0)]),
            components: Box::new([std::ptr::null_mut()]),
            capacity: 0,
            entities: vec![],
        }
    }

    pub fn alloc(&mut self, cap: usize) {
        use std::alloc::{alloc, handle_alloc_error};

        if cap == 0 {
            return;
        }

        for (type_info, components_ptr) in
            self.component_types.iter().zip(self.components.iter_mut())
        {
            let Ok(layout) =
                Layout::from_size_align(cap * type_info.layout.size(), type_info.layout.align())
            else {
                continue;
            };

            let ptr = unsafe { alloc(layout) };

            if ptr.is_null() {
                handle_alloc_error(layout);
            }

            *components_ptr = ptr;
        }

        self.capacity = cap;
    }

    pub fn realloc(&mut self) {
        use std::alloc::{handle_alloc_error, realloc};

        let next_capacity = 3 * (self.capacity + 1) / 2;

        for (type_info, components_ptr) in
            self.component_types.iter().zip(self.components.iter_mut())
        {
            let Ok(prev_layout) = Layout::from_size_align(
                self.capacity * type_info.layout.size(),
                type_info.layout.align(),
            ) else {
                continue;
            };

            let ptr = unsafe {
                realloc(
                    *components_ptr,
                    prev_layout,
                    next_capacity * type_info.layout.size(),
                )
            };

            if ptr.is_null() {
                handle_alloc_error(prev_layout);
            }

            *components_ptr = ptr;
        }

        self.capacity = next_capacity;
    }

    pub fn add_component<C: Component>(&mut self, value: C) {
        assert_eq!(self.component_types.len(), 1);

        if self.capacity == 0 {
            self.alloc(1);
        } else if self.entities.len() == self.capacity {
            self.realloc();
        }

        let index = self.index[&TypeId::of::<C>()];
        let ptr = self.components[index].cast::<C>();

        unsafe {
            ptr.add(self.entities.len()).write(value);
        }
    }
}

impl Drop for Archetype {
    fn drop(&mut self) {
        use std::alloc::dealloc;

        for (i, &components_ptr) in self.components.iter().enumerate() {
            let elem_layout = self.component_types[i].layout;
            let size = elem_layout.size();
            let drop = self.component_types[i].drop;

            let Ok(layout) =
                Layout::from_size_align(self.capacity * elem_layout.size(), elem_layout.align())
            else {
                continue;
            };

            for j in 0..self.entities.len() {
                unsafe {
                    drop(components_ptr.add(size * j));
                }
            }

            unsafe {
                dealloc(components_ptr, layout);
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

        let entity_index = self.archetypes[archetype_index].entities.len();
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
        let components_len = self.archetypes[archetype_index].entities.len();
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

        #[derive(Debug, PartialEq)]
        struct Tag;
        impl Component for Tag {}

        let mut world = World::default();

        let entities = [
            world.spawn(Name(String::from("First"))),
            world.spawn(Name(String::from("Second"))),
            world.spawn(Speed(42.0)),
            world.spawn(Name(String::from("Third"))),
            world.spawn(Speed(69.0)),
            world.spawn(Tag),
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
        );

        assert_eq!(
            world.query_mut::<Tag>().collect::<Vec<_>>(),
            [(entities[5], &mut Tag)],
        );
    }
}
