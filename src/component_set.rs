use crate::{archetype::TypeInfo, archetype::Archetype, Component};
use smallvec::SmallVec;
use std::{any::TypeId, collections::HashMap};

pub(crate) const N_STACK_TYPE_IDS: usize = 32;

/// Represents a pack of components. Does not intended to implement it by hand.
///
/// # Safety
///
/// - new entity should be added immediately after `write_archetype`.
/// - `component_infos` should sort `TypeInfo`s by their ids.
pub unsafe trait ComponentSet: Sized + 'static {
    /// The number of components inside this pack.
    const COMPONENT_COUNT: usize;

    /// # Safety
    ///
    /// New entity should be added immediately after this call.
    unsafe fn write_archetype(self, archetype: &mut Archetype);

    /// The information about each type in this type pack. Should be sorted by id.
    fn component_infos() -> impl AsRef<[TypeInfo]>;

    /// Gives an index at which a given component pack lies.
    fn get_index(index: &HashMap<Box<[TypeId]>, usize>) -> Option<usize> {
        let ids: SmallVec<[TypeId; N_STACK_TYPE_IDS]> = Self::component_infos()
            .as_ref()
            .iter()
            .map(|info| info.id)
            .collect();

        index.get(&ids[..]).copied()
    }

    /// Similar to `component_infos` but contains only IDs.
    fn component_ids() -> Box<[TypeId]> {
        Self::component_infos()
            .as_ref()
            .iter()
            .map(|info| info.id)
            .collect()
    }

    /// Creates an archetype based on this component pack.
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
        unsafe { (self,).write_archetype(archetype) };
    }

    fn component_infos() -> impl AsRef<[TypeInfo]> {
        <(T,) as ComponentSet>::component_infos()
    }
}

#[doc(hidden)]
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
                    unsafe { archetype.write_to_end($t) };
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
