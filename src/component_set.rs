use crate::{archetype::TypeInfo, Archetype, Component};
use std::{any::TypeId, collections::HashMap};

pub unsafe trait ComponentSet: Sized + 'static {
    const COMPONENT_COUNT: usize;

    unsafe fn write_archetype(self, archetype: &mut Archetype);

    fn get_index(index: &HashMap<Box<[TypeId]>, usize>) -> Option<usize>;

    fn component_ids() -> Box<[TypeId]>;

    fn component_type_infos() -> Box<[TypeInfo]>;

    fn make_archetype() -> Archetype {
        let types = Self::component_type_infos();

        Archetype {
            capacity: 0,
            index: HashMap::from_iter(types.iter().map(|t| t.id).zip(0..)),
            components: vec![std::ptr::null_mut(); types.len()].into(),
            entities: vec![],
            component_types: types,
        }
    }
}

unsafe impl<T: Component> ComponentSet for T {
    const COMPONENT_COUNT: usize = <(T,) as ComponentSet>::COMPONENT_COUNT;

    unsafe fn write_archetype(self, archetype: &mut Archetype) {
        (self,).write_archetype(archetype);
    }

    fn get_index(index: &HashMap<Box<[TypeId]>, usize>) -> Option<usize> {
        <(T,) as ComponentSet>::get_index(index)
    }

    fn component_ids() -> Box<[TypeId]> {
        <(T,) as ComponentSet>::component_ids()
    }

    fn component_type_infos() -> Box<[TypeInfo]> {
        <(T,) as ComponentSet>::component_type_infos()
    }
}

macro_rules! impl_tuple_component_set {
    ( @~count ) => { 0 };

    ( @~count $T:ident $( $Tail:ident )* ) => {{
        1 + impl_tuple_component_set!( @~count $( $Tail )* )
    }};

    ( ( $( $t:ident : $T:ident ),+) ) => {
        unsafe impl< $( $T: Component, )+ > ComponentSet for ( $( $T, )+ ) {
            const COMPONENT_COUNT: usize = impl_tuple_component_set!( @~count $( $T )+ );

            unsafe fn write_archetype(self, archetype: &mut Archetype) {
                archetype.reserve(Self::COMPONENT_COUNT);
                let ( $( $t, )+ ) = self;

                $(
                    archetype.write_to_end($t);
                )+
            }

            fn get_index(index: &HashMap<Box<[TypeId]>, usize>) -> Option<usize> {
                let mut ids = [
                    $(
                        TypeId::of::< $T >(),
                    )+
                ];
                ids.sort_unstable();

                index.get(&ids[..]).copied()
            }

            fn component_ids() -> Box<[TypeId]> {
                let mut ids = Box::new([
                    $(
                        TypeId::of::< $T >(),
                    )+
                ]);
                ids.sort_unstable();
                ids
            }

            fn component_type_infos() -> Box<[TypeInfo]> {
                let mut ids = Box::new([
                    $(
                        TypeInfo::of::< $T >(),
                    )+
                ]);
                ids.sort_unstable_by_key(|info| info.id);
                ids
            }
        }
    };
}

impl_tuple_component_set! { (a: A) }
impl_tuple_component_set! { (a: A, b: B) }
impl_tuple_component_set! { (a: A, b: B, c: C) }
impl_tuple_component_set! { (a: A, b: B, c: C, d: D) }
impl_tuple_component_set! { (a: A, b: B, c: C, d: D, e: E) }
impl_tuple_component_set! { (a: A, b: B, c: C, d: D, e: E, f: F) }
impl_tuple_component_set! { (a: A, b: B, c: C, d: D, e: E, f: F, g: G) }
impl_tuple_component_set! { (a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H) }
impl_tuple_component_set! { (a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I) }
impl_tuple_component_set! { (a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J) }
impl_tuple_component_set! { (a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J, k: K) }
impl_tuple_component_set! { (a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J, k: K, l: L) }
impl_tuple_component_set! { (a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J, k: K, l: L, m: M) }
impl_tuple_component_set! { (a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J, k: K, l: L, m: M, n: N) }
