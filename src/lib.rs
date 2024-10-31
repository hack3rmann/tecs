#![allow(dead_code, clippy::missing_safety_doc)]

mod archetype;
mod component_set;
mod simple;
mod world;

pub use archetype::Archetype;
pub use component_set::ComponentSet;
pub use world::{Component, World};

type Entity = u32;

pub(crate) struct Location {
    pub entity_index: u32,
    pub archetype_index: u32,
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
            [(entities[0], &mut Age(18)), (entities[1], &mut Age(24)),],
        );
    }
}
