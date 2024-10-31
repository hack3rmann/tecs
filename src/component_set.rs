use crate::{archetype::TypeInfo, Archetype, Component};
use smallvec::SmallVec;
use std::{any::TypeId, collections::HashMap};

pub const N_STACK_TYPE_IDS: usize = 32;

pub unsafe trait ComponentSet: Sized + 'static {
    const COMPONENT_COUNT: usize;

    unsafe fn write_archetype(self, archetype: &mut Archetype);

    fn component_infos() -> impl AsRef<[TypeInfo]>;

    fn get_index(index: &HashMap<Box<[TypeId]>, usize>) -> Option<usize> {
        let ids: SmallVec<[TypeId; N_STACK_TYPE_IDS]> = Self::component_infos()
            .as_ref()
            .iter()
            .map(|info| info.id)
            .collect();

        index.get(&ids[..]).copied()
    }

    fn component_ids() -> Box<[TypeId]> {
        Self::component_infos()
            .as_ref()
            .iter()
            .map(|info| info.id)
            .collect()
    }

    fn make_archetype() -> Archetype {
        let types: Box<[TypeInfo]> = Self::component_infos().as_ref().to_owned().into();

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

    fn component_infos() -> impl AsRef<[TypeInfo]> {
        <(T,) as ComponentSet>::component_infos()
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

            fn component_infos() -> impl AsRef<[TypeInfo]> {
                let mut ids = [
                    $(
                        TypeInfo::of::< $T >(),
                    )+
                ];
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
