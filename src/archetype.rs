use crate::{Component, EntityId};
use std::{alloc::Layout, any::TypeId, collections::HashMap};

#[derive(Clone, Debug)]
pub struct TypeInfo {
    pub(crate) id: TypeId,
    pub(crate) layout: Layout,
    pub(crate) drop: unsafe fn(*mut u8),
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

/// A group of components that are used together in entities.
///
/// # Note
///
/// Does not intended to use it by hand.
#[derive(Debug)]
pub struct Archetype {
    pub(crate) index: HashMap<TypeId, usize>,
    pub(crate) component_types: Box<[TypeInfo]>,
    pub(crate) components: Box<[*mut u8]>,
    pub(crate) capacity: usize,
    pub(crate) entities: Vec<EntityId>,
}

impl Archetype {
    /// Checks component `C` in this archetype.
    pub fn contains<C: Component>(&self) -> bool {
        self.component_types
            .binary_search(&TypeInfo::of::<C>())
            .is_ok()
    }

    // pub fn contains_all(&self, components: impl IntoIterator<Item = TypeId>) -> bool {
    //     components.into_iter().all(|c| {
    //         self.component_types
    //             .binary_search_by_key(&c, |info| info.id)
    //             .is_ok()
    //     })
    // }

    pub(crate) fn alloc(&mut self, cap: usize) {
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

    pub(crate) fn reserve(&mut self, min_additional_capacity: usize) {
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

    pub(crate) unsafe fn write_to_end<C: Component>(&mut self, value: C) {
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

        if self.capacity == 0 {
            return;
        }

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
