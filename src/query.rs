use core::slice;
use std::any::TypeId;

use crate::{Component, EntityId, World};

pub trait ComponentRef<'t>: Sized + 't {
    type Value: Component;

    fn from_mut(value: &'t mut Self::Value) -> Self;
}

impl<'t, T: Component> ComponentRef<'t> for &'t T {
    type Value = T;

    fn from_mut(value: &'t mut Self::Value) -> Self {
        value
    }
}

impl<'t, T: Component> ComponentRef<'t> for &'t mut T {
    type Value = T;

    fn from_mut(value: &'t mut Self::Value) -> Self {
        value
    }
}

pub trait Query<'w>: Sized + 'w {
    type Output: 'w;

    fn query(world: &'w World) -> impl Iterator<Item = Self::Output> + 'w;
}

pub trait QueryMut<'w>: Sized + 'w {
    type Output: 'w;

    fn query_mut(world: &'w mut World) -> impl Iterator<Item = Self::Output> + 'w;
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

macro_rules! impl_query {
    ( @map ) => {
        |x| (x,)
    };
    ( @map $A:ident ) => {
        |(x, y)| (x, y)
    };
    ( @map $A:ident $B:ident ) => {
        |((x, y), z)| (x, y, z)
    };
    ( @map $A:ident $B:ident $C:ident ) => {
        |(((x, y), z), w)| (x, y, z, w)
    };
    ( @map $A:ident $B:ident $C:ident $D:ident ) => {
        |((((a, b), c), d), e)| (a, b, c, d, e)
    };
    ( @map $A:ident $B:ident $C:ident $D:ident $E:ident ) => {
        |(((((a, b), c), d), e), f)| (a, b, c, d, e, f)
    };
    ( @map $A:ident $B:ident $C:ident $D:ident $E:ident $F:ident ) => {
        |((((((a, b), c), d), e), f), g)| (a, b, c, d, e, f, g)
    };
    ( @map $A:ident $B:ident $C:ident $D:ident $E:ident $F:ident $G:ident ) => {
        |(((((((a, b), c), d), e), f), g), h)| (a, b, c, d, e, f, g, h)
    };
    ( $T:ident $( $Tail:ident )* ) => {
        impl<'w, $T: Component, $( $Tail: Component, )* > Query<'w> for (EntityId, &'w $T, $( &'w $Tail, )* ) {
            type Output = Self;

            fn query(world: &'w World) -> impl Iterator<Item = Self::Output> + 'w {
                world
                    .archetypes
                    .iter()
                    .filter(move |arch| {
                        !arch.entities.is_empty()
                            && arch.contains::< $T >()
                            $(
                                && arch.contains::< $Tail >()
                            )*
                    })
                    .flat_map(move |arch| {
                        arch.entities
                            .iter()
                            .copied()
                            .zip({
                                let components_index = arch.index[&TypeId::of::< $T >()];
                                let mut ptr = arch.components[components_index].cast::< $T >();

                                if ptr.is_null() {
                                    ptr = std::ptr::NonNull::< $T >::dangling().as_ptr();
                                }

                                unsafe { slice::from_raw_parts(ptr, arch.entities.len()) }
                            })
                            $(
                                .zip({
                                    let components_index = arch.index[&TypeId::of::< $Tail >()];
                                    let mut ptr = arch.components[components_index].cast::< $Tail >();

                                    if ptr.is_null() {
                                        ptr = std::ptr::NonNull::< $Tail >::dangling().as_ptr();
                                    }

                                    unsafe { slice::from_raw_parts(ptr, arch.entities.len()) }
                                })
                            )*
                            .map(impl_query!(@map $T $( $Tail )* ))
                    })
            }
        }

        impl<'w, $T: Component, $( $Tail: Component, )* > Query<'w> for (&'w $T, $( &'w $Tail, )* ) {
            type Output = Self;

            fn query(world: &'w World) -> impl Iterator<Item = Self::Output> + 'w {
                world
                    .archetypes
                    .iter()
                    .filter(move |arch| {
                        !arch.entities.is_empty()
                            && arch.contains::< $T >()
                            $(
                                && arch.contains::< $Tail >()
                            )*
                    })
                    .flat_map(move |arch| {
                        {
                            let components_index = arch.index[&TypeId::of::< $T >()];
                            let mut ptr = arch.components[components_index].cast::< $T >();

                            if ptr.is_null() {
                                ptr = std::ptr::NonNull::< $T >::dangling().as_ptr();
                            }

                            unsafe { slice::from_raw_parts(ptr, arch.entities.len()) }
                        }.into_iter()
                            $(
                                .zip({
                                    let components_index = arch.index[&TypeId::of::< $Tail >()];
                                    let mut ptr = arch.components[components_index].cast::< $Tail >();

                                    if ptr.is_null() {
                                        ptr = std::ptr::NonNull::< $Tail >::dangling().as_ptr();
                                    }

                                    unsafe { slice::from_raw_parts(ptr, arch.entities.len()) }
                                })
                            )*
                            .map(impl_query!(@map $( $Tail )* ))
                    })
            }
        }

        impl<'w, $T: ComponentRef<'w>, $( $Tail: ComponentRef<'w>, )*> QueryMut<'w> for (EntityId, $T, $( $Tail, )*) {
            type Output = Self;

            fn query_mut(world: &'w mut World) -> impl Iterator<Item = Self::Output> + 'w {
                world
                    .archetypes
                    .iter_mut()
                    .filter(move |arch| {
                        !arch.entities.is_empty()
                            && arch.contains::< $T ::Value>()
                            $(
                                && arch.contains::< $Tail ::Value>()
                            )*
                    })
                    .flat_map(move |arch| {
                        arch.entities
                            .iter()
                            .copied()
                            .zip(
                                {
                                    let components_index = arch.index[&TypeId::of::< $T ::Value>()];
                                    let mut ptr = arch.components[components_index].cast::< $T ::Value>();

                                    if ptr.is_null() {
                                        ptr = std::ptr::NonNull::< $T ::Value>::dangling().as_ptr();
                                    }

                                    unsafe { slice::from_raw_parts_mut(ptr, arch.entities.len()) }
                                }
                                .iter_mut()
                                .map( $T ::from_mut),
                            )
                            $(
                                .zip(
                                    {
                                        let components_index = arch.index[&TypeId::of::< $Tail ::Value>()];
                                        let mut ptr = arch.components[components_index].cast::< $Tail ::Value>();

                                        if ptr.is_null() {
                                            ptr = std::ptr::NonNull::< $Tail ::Value>::dangling().as_ptr();
                                        }

                                        unsafe { slice::from_raw_parts_mut(ptr, arch.entities.len()) }
                                    }
                                    .iter_mut()
                                    .map( $Tail ::from_mut),
                                )
                            )*
                            .map(impl_query!(@map $T $( $Tail )* ))
                    })
            }
        }

        impl<'w, $T: ComponentRef<'w>, $( $Tail: ComponentRef<'w>, )*> QueryMut<'w> for ($T, $( $Tail, )*) {
            type Output = Self;

            fn query_mut(world: &'w mut World) -> impl Iterator<Item = Self::Output> + 'w {
                world
                    .archetypes
                    .iter_mut()
                    .filter(move |arch| {
                        !arch.entities.is_empty()
                            && arch.contains::< $T ::Value>()
                            $(
                                && arch.contains::< $Tail ::Value>()
                            )*
                    })
                    .flat_map(move |arch| {
                            {
                                let components_index = arch.index[&TypeId::of::< $T ::Value>()];
                                let mut ptr = arch.components[components_index].cast::< $T ::Value>();

                                if ptr.is_null() {
                                    ptr = std::ptr::NonNull::< $T ::Value>::dangling().as_ptr();
                                }

                                unsafe { slice::from_raw_parts_mut(ptr, arch.entities.len()) }
                            }
                            .iter_mut()
                            .map( $T ::from_mut)
                            $(
                                .zip(
                                    {
                                        let components_index = arch.index[&TypeId::of::< $Tail ::Value>()];
                                        let mut ptr = arch.components[components_index].cast::< $Tail ::Value>();

                                        if ptr.is_null() {
                                            ptr = std::ptr::NonNull::< $Tail ::Value>::dangling().as_ptr();
                                        }

                                        unsafe { slice::from_raw_parts_mut(ptr, arch.entities.len()) }
                                    }
                                    .iter_mut()
                                    .map( $Tail ::from_mut),
                                )
                            )*
                            .map(impl_query!(@map $( $Tail )* ))
                    })
            }
        }
    };
}

impl_query! { A }
impl_query! { A B }
impl_query! { A B C }
impl_query! { A B C D }
impl_query! { A B C D E }
impl_query! { A B C D E F }
impl_query! { A B C D E F G }
