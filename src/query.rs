use core::slice;
use std::any::TypeId;

use crate::{Component, Entity, World};

pub trait Query<'w>: Sized + 'w {
    type Output: 'w;

    fn query(world: &'w World) -> impl Iterator<Item = Self::Output> + 'w;
}

pub trait QueryMut<'w>: Sized + 'w {
    type Output: 'w;

    fn query_mut(world: &'w mut World) -> impl Iterator<Item = Self::Output> + 'w;
}

impl<'w, T: Component> Query<'w> for (Entity, &'w T) {
    type Output = Self;

    fn query(world: &'w World) -> impl Iterator<Item = Self::Output> + 'w {
        world
            .archetypes
            .iter()
            .filter(move |arch| !arch.entities.is_empty() && arch.contains::<T>())
            .flat_map(move |arch| {
                let components_index = arch.index[&TypeId::of::<T>()];
                let mut ptr = arch.components[components_index].cast::<T>();

                if ptr.is_null() {
                    ptr = std::ptr::NonNull::<T>::dangling().as_ptr();
                }

                let components = unsafe { slice::from_raw_parts(ptr, arch.entities.len()) };

                arch.entities.iter().copied().zip(components)
            })
    }
}

impl<'w, T: Component> Query<'w> for &'w T {
    type Output = Self;

    fn query(world: &'w World) -> impl Iterator<Item = Self::Output> + 'w {
        world
            .archetypes
            .iter()
            .filter(move |arch| !arch.entities.is_empty() && arch.contains::<T>())
            .flat_map(move |arch| {
                let components_index = arch.index[&TypeId::of::<T>()];
                let mut ptr = arch.components[components_index].cast::<T>();

                if ptr.is_null() {
                    ptr = std::ptr::NonNull::<T>::dangling().as_ptr();
                }

                let components = unsafe { slice::from_raw_parts(ptr, arch.entities.len()) };

                components.iter()
            })
    }
}

impl<'w, T: Component> QueryMut<'w> for (Entity, &'w mut T) {
    type Output = Self;

    fn query_mut(world: &'w mut World) -> impl Iterator<Item = Self::Output> + 'w {
        world
            .archetypes
            .iter_mut()
            .filter(move |arch| !arch.entities.is_empty() && arch.contains::<T>())
            .flat_map(move |arch| {
                let components_index = arch.index[&TypeId::of::<T>()];
                let mut ptr = arch.components[components_index].cast::<T>();

                if ptr.is_null() {
                    ptr = std::ptr::NonNull::<T>::dangling().as_ptr();
                }

                let components = unsafe { slice::from_raw_parts_mut(ptr, arch.entities.len()) };

                arch.entities.iter().copied().zip(components)
            })
    }
}

impl<'w, T: Component> QueryMut<'w> for &'w mut T {
    type Output = Self;

    fn query_mut(world: &'w mut World) -> impl Iterator<Item = Self::Output> + 'w {
        world
            .archetypes
            .iter_mut()
            .filter(move |arch| !arch.entities.is_empty() && arch.contains::<T>())
            .flat_map(move |arch| {
                let components_index = arch.index[&TypeId::of::<T>()];
                let mut ptr = arch.components[components_index].cast::<T>();

                if ptr.is_null() {
                    ptr = std::ptr::NonNull::<T>::dangling().as_ptr();
                }

                let components = unsafe { slice::from_raw_parts_mut(ptr, arch.entities.len()) };

                components.iter_mut()
            })
    }
}
