#![allow(dead_code, clippy::missing_safety_doc)]

mod archetype;
mod component_set;
mod query;
mod simple;
mod world;

pub use archetype::Archetype;
pub use component_set::ComponentSet;
pub use world::{Component, World};

type EntityId = u32;

#[derive(Clone, Debug, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
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
            world.query::<(EntityId, &Name)>().collect::<Vec<_>>(),
            [
                (entities[0], &Name(String::from("First"))),
                (entities[1], &Name(String::from("Second"))),
                (entities[3], &Name(String::from("Third"))),
            ],
        );

        assert_eq!(
            world.query::<(EntityId, &Speed)>().collect::<Vec<_>>(),
            [(entities[2], &Speed(42.0)), (entities[4], &Speed(69.0)),],
        );

        assert_eq!(
            world.query::<(EntityId, &Tag)>().collect::<Vec<_>>(),
            [(entities[5], &Tag)],
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
            world.query::<(EntityId, &Name)>().collect::<Vec<_>>(),
            [
                (entities[0], &Name(String::from("John"))),
                (entities[1], &Name(String::from("Hannah"))),
                (entities[2], &Name(String::from("Bob"))),
            ],
        );

        assert_eq!(
            world.query::<(EntityId, &Age)>().collect::<Vec<_>>(),
            [(entities[0], &Age(18)), (entities[1], &mut Age(24)),],
        );
    }

    #[test]
    fn three_components() {
        #[derive(Debug, PartialEq)]
        struct Name(String);
        impl Component for Name {}

        #[derive(Debug, PartialEq)]
        struct Age(u32);
        impl Component for Age {}

        #[derive(Debug, PartialEq)]
        struct Height(f32);
        impl Component for Height {}

        let mut world = World::default();

        let entities = [
            world.spawn((Name(String::from("John")), Age(18))),
            world.spawn((Name(String::from("Hannah")), Age(24))),
            world.spawn((Name(String::from("Bob")), Height(160.0))),
            world.spawn((Name(String::from("Alice")), Height(200.0), Age(19))),
        ];

        assert_eq!(
            world.query::<(EntityId, &Name, &Age)>().collect::<Vec<_>>(),
            [
                (entities[0], &Name(String::from("John")), &Age(18)),
                (entities[1], &Name(String::from("Hannah")), &Age(24)),
                (entities[3], &Name(String::from("Alice")), &Age(19)),
            ],
        );

        assert_eq!(
            world.query::<(EntityId, &Name, &Height)>().collect::<Vec<_>>(),
            [
                (entities[2], &Name(String::from("Bob")), &Height(160.0)),
                (entities[3], &Name(String::from("Alice")), &Height(200.0)),
            ],
        );

        assert_eq!(
            world.query::<(EntityId, &Age, &Height)>().collect::<Vec<_>>(),
            [(entities[3], &Age(19), &Height(200.0)),],
        );
    }

    #[test]
    fn gets() {
        #[derive(Debug, PartialEq)]
        struct Name(&'static str);
        impl Component for Name {}

        #[derive(Debug, PartialEq)]
        struct Age(u32);
        impl Component for Age {}

        let mut world = World::default();

        let id = world.spawn((Name("John"), Age(99)));
        let entity = world.entity(id);

        assert_eq!(entity.get::<Name>(), Some(&Name("John")));
        assert_eq!(entity.get::<Age>(), Some(&Age(99)));
    }
}
