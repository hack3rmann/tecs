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

    #[derive(Debug, PartialEq)]
    struct Name(&'static str);
    impl Component for Name {}

    #[derive(Debug, PartialEq)]
    struct Age(u32);
    impl Component for Age {}

    #[derive(Debug, PartialEq)]
    struct Height(f32);
    impl Component for Height {}

    #[derive(Debug, PartialEq)]
    struct Speed(f32);
    impl Component for Speed {}

    #[derive(Debug, PartialEq)]
    struct Tag;
    impl Component for Tag {}

    #[test]
    fn single_component_archetype() {
        let mut world = World::default();

        let entities = [
            world.spawn(Name("First")),
            world.spawn(Name("Second")),
            world.spawn(Speed(42.0)),
            world.spawn(Name("Third")),
            world.spawn(Speed(69.0)),
            world.spawn(Tag),
        ];

        assert_eq!(
            world.query::<(EntityId, &Name)>().collect::<Vec<_>>(),
            [
                (entities[0], &Name("First")),
                (entities[1], &Name("Second")),
                (entities[3], &Name("Third")),
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
        let mut world = World::default();

        let entities = [
            world.spawn((Name("John"), Age(18))),
            world.spawn((Name("Hannah"), Age(24))),
            world.spawn(Name("Bob")),
        ];

        assert_eq!(
            world.query::<(EntityId, &Name)>().collect::<Vec<_>>(),
            [
                (entities[0], &Name("John")),
                (entities[1], &Name("Hannah")),
                (entities[2], &Name("Bob")),
            ],
        );

        assert_eq!(
            world.query::<(EntityId, &Age)>().collect::<Vec<_>>(),
            [(entities[0], &Age(18)), (entities[1], &mut Age(24)),],
        );
    }

    #[test]
    fn three_components() {
        let mut world = World::default();

        let entities = [
            world.spawn((Name("John"), Age(18))),
            world.spawn((Name("Hannah"), Age(24))),
            world.spawn((Name("Bob"), Height(160.0))),
            world.spawn((Name("Alice"), Height(200.0), Age(19))),
        ];

        assert_eq!(
            world.query::<(EntityId, &Name, &Age)>().collect::<Vec<_>>(),
            [
                (entities[0], &Name("John"), &Age(18)),
                (entities[1], &Name("Hannah"), &Age(24)),
                (entities[3], &Name("Alice"), &Age(19)),
            ],
        );

        assert_eq!(
            world
                .query::<(EntityId, &Name, &Height)>()
                .collect::<Vec<_>>(),
            [
                (entities[2], &Name("Bob"), &Height(160.0)),
                (entities[3], &Name("Alice"), &Height(200.0)),
            ],
        );

        assert_eq!(
            world
                .query::<(EntityId, &Age, &Height)>()
                .collect::<Vec<_>>(),
            [(entities[3], &Age(19), &Height(200.0)),],
        );
    }

    #[test]
    fn gets() {
        let mut world = World::default();

        let id = world.spawn((Name("John"), Age(99)));
        let entity = world.entity(id);

        assert_eq!(entity.get::<Name>(), Some(&Name("John")));
        assert_eq!(entity.get::<Age>(), Some(&Age(99)));
    }

    #[test]
    fn mut_query_shared() {
        let mut world = World::default();

        world.spawn((Name("Kristie"), Height(67.0), Age(9)));
        world.spawn((Name("Jordan"), Height(89.0), Age(10)));
        world.spawn((Name("Bob"), Height(101.0), Age(11)));
        world.spawn((Name("Michael"), Height(42.0), Age(12)));
        world.spawn((Name("Dave"), Height(34.0), Age(13)));
        world.spawn((Name("Paul"), Height(890.0), Age(14)));
        world.spawn((Name("Joseph"), Height(67.0), Age(15)));
        world.spawn((Name("Alex"), Height(67.0), Age(16)));
        world.spawn((Name("Steve"), Height(67.0), Age(17)));

        for (_entity, name, _age) in world.query_mut::<(EntityId, &mut Name, &Age)>() {
            *name = Name("None");
        }

        assert!(world.query_mut::<(&Name,)>().all(|(name,)| name == &Name("None")));
    }
}
