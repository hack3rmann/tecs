#![allow(dead_code, clippy::missing_safety_doc)]

mod simple;

use std::{alloc::Layout, any::TypeId, collections::HashMap, slice};

type Entity = u32;

pub struct TypeInfo {
    pub id: TypeId,
    pub layout: Layout,
    pub drop: unsafe fn(*mut u8),
}

impl std::cmp::PartialEq for TypeInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl std::cmp::Eq for TypeInfo {}

impl std::cmp::PartialOrd for TypeInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for TypeInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
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

    pub fn contains<C: Component>(&self) -> bool {
        self.component_types.contains(&TypeInfo::of::<C>())
    }

    pub fn alloc(&mut self, cap: usize) {
        use std::alloc::{alloc, handle_alloc_error};

        if cap == 0 {
            return;
        }

        for (type_info, components_ptr) in
            self.component_types.iter().zip(self.components.iter_mut())
        {
            if type_info.layout.size() == 0 {
                continue;
            }

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

    pub fn reserve(&mut self, min_additional_capacity: usize) {
        use std::alloc::{handle_alloc_error, realloc};

        if self.capacity - self.entities.len() >= min_additional_capacity {
            return;
        }

        if self.capacity == 0 {
            self.alloc(min_additional_capacity);
            return;
        }

        let additional_capacity = min_additional_capacity.max(self.capacity / 2 + 1);
        let next_capacity = self.capacity + additional_capacity;

        for (type_info, components_ptr) in
            self.component_types.iter().zip(self.components.iter_mut())
        {
            if type_info.layout.size() == 0 {
                continue;
            }

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

            if components_ptr.is_null() {
                continue;
            }

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

pub trait ComponentSet: Sized + 'static {
    unsafe fn write_archetype(self, archetype: &mut Archetype);
    fn get_index(index: &HashMap<Box<[TypeId]>, usize>) -> Option<usize>;
    fn set_types() -> Box<[TypeId]>;
    fn make_archetype() -> Archetype;
}

impl<T: Component> ComponentSet for T {
    unsafe fn write_archetype(self, archetype: &mut Archetype) {
        archetype.reserve(1);

        let index = archetype.index[&TypeId::of::<T>()];
        let ptr = archetype.components[index].cast::<T>();

        unsafe {
            ptr.add(archetype.entities.len()).write(self);
        }
    }

    fn get_index(index: &HashMap<Box<[TypeId]>, usize>) -> Option<usize> {
        index.get(&[TypeId::of::<T>()][..]).copied()
    }

    fn set_types() -> Box<[TypeId]> {
        Box::new([TypeId::of::<T>()])
    }

    fn make_archetype() -> Archetype {
        Archetype::new::<T>()
    }
}

impl<T: Component, S: Component> ComponentSet for (T, S) {
    unsafe fn write_archetype(self, archetype: &mut Archetype) {
        archetype.reserve(2);

        let index = archetype.index[&TypeId::of::<T>()];
        let ptr = archetype.components[index].cast::<T>();

        unsafe {
            ptr.add(archetype.entities.len()).write(self.0);
        }

        let index = archetype.index[&TypeId::of::<S>()];
        let ptr = archetype.components[index].cast::<S>();

        unsafe {
            ptr.add(archetype.entities.len()).write(self.1);
        }
    }

    fn get_index(index: &HashMap<Box<[TypeId]>, usize>) -> Option<usize> {
        let mut ids = [TypeId::of::<T>(), TypeId::of::<S>()];
        ids.sort_unstable();

        index.get(&ids[..]).copied()
    }

    fn set_types() -> Box<[TypeId]> {
        let mut ids = Box::new([TypeId::of::<T>(), TypeId::of::<S>()]);
        ids.sort_unstable();
        ids
    }

    fn make_archetype() -> Archetype {
        let mut ids = [TypeInfo::of::<T>(), TypeInfo::of::<S>()];
        ids.sort_unstable_by_key(|x| x.id);

        Archetype {
            capacity: 0,
            index: HashMap::from([(ids[0].id, 0), (ids[1].id, 1)]),
            component_types: Box::new(ids),
            components: Box::new([std::ptr::null_mut(), std::ptr::null_mut()]),
            entities: vec![],
        }
    }
}

impl World {
    pub fn spawn<S: ComponentSet>(&mut self, set: S) -> Entity {
        let entity = self.locations.len() as Entity;

        let archetype_index = match S::get_index(&self.index) {
            Some(index) => index,
            None => {
                let archetype = S::make_archetype();
                let index = self.archetypes.len();

                self.index.insert(S::set_types(), index);
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
        unsafe {
            set.write_archetype(&mut self.archetypes[archetype_index]);
        }
        self.archetypes[archetype_index].entities.push(entity);

        entity
    }

    pub fn query_mut<C: Component>(&mut self) -> impl Iterator<Item = (Entity, &mut C)> {
        self.archetypes.iter_mut()
            .filter(|arch| !arch.entities.is_empty() && arch.contains::<C>())
            .flat_map(|arch| {
                let components_index = arch.index[&TypeId::of::<C>()];
                let mut ptr = arch.components[components_index].cast::<C>();

                if ptr.is_null() {
                    ptr = std::ptr::NonNull::<C>::dangling().as_ptr();
                }

                let components = unsafe {
                    slice::from_raw_parts_mut(ptr, arch.entities.len())
                };

                arch.entities.iter().copied().zip(components)
            })
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

    #[test]
    fn two_components() {
        #[derive(Debug, PartialEq)]
        struct Name(String);
        impl Component for Name {}

        #[derive(Debug, PartialEq)]
        struct Age(u32);
        impl Component for Age {}

        let mut world = World::default();

        let entities = [
            world.spawn((Name(String::from("John")), Age(18))),
            world.spawn((Name(String::from("Hannah")), Age(24))),
            world.spawn(Name(String::from("Bob"))),
        ];

        assert_eq!(
            world.query_mut::<Name>().collect::<Vec<_>>(),
            [
                (entities[0], &mut Name(String::from("John"))),
                (entities[1], &mut Name(String::from("Hannah"))),
                (entities[2], &mut Name(String::from("Bob"))),
            ],
        );

        assert_eq!(
            world.query_mut::<Age>().collect::<Vec<_>>(),
            [
                (entities[0], &mut Age(18)),
                (entities[1], &mut Age(24)),
            ],
        );
    }
}
