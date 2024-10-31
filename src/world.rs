use crate::{
    query::{Query, QueryMut},
    archetype::Archetype, component_set::ComponentSet, EntityId, Location,
};
use std::{any::TypeId, collections::HashMap};

/// An ECS world. The place where each component and entity are stored.
///
/// # Example
///
/// Spawning an antity and retrieving its components.
///
/// ```rust
/// use tecs::{World, Component};
///
/// #[derive(Debug, PartialEq)]
/// struct Name(&'static str);
/// impl Component for Name {}
///
/// #[derive(Debug, PartialEq)]
/// struct Age(u32);
/// impl Component for Age {}
///
/// let mut world = World::new();
///
/// let id = world.spawn((Name("Marcus"), Age(21)));
/// let entity = world.entity(id);
///
/// assert_eq!(entity.get::<Name>(), Some(&Name("Marcus")));
/// assert_eq!(entity.get::<Age>(), Some(&Age(21)));
/// ```
#[derive(Default)]
pub struct World {
    pub(crate) archetypes: Vec<Archetype>,
    pub(crate) locations: Vec<Location>,
    pub(crate) index: HashMap<Box<[TypeId]>, usize>,
}

/// Signifies that given type can be used as a component.
///
/// # Example
///
/// Implementing [`Component`].
///
/// ```rust
/// use tecs::Component;
///
/// // Can be implemented on ZSTs
/// struct CanFly;
/// impl Component for CanFly {}
///
/// // .. or any type that is `Sized` and `'static`
/// struct Velocity([f32; 3]);
/// impl Component for Velocity {}
/// ```
pub trait Component: Sized + 'static {}

impl World {
    /// Constructs new empty world.
    pub fn new() -> Self {
        Self::default()
    }

    /// Spawns an entity with given components and returns its id.
    ///
    /// # Example
    ///
    /// Spawning an entity with multiple components.
    ///
    /// ```rust
    /// use tecs::{World, Component};
    ///
    /// // Can be implemented on ZSTs
    /// struct CanFly;
    /// impl Component for CanFly {}
    ///
    /// struct CanJump;
    /// impl Component for CanJump {}
    ///
    /// let mut world = World::new();
    ///
    /// let id = world.spawn((CanFly, CanJump));
    /// ```
    pub fn spawn<S: ComponentSet>(&mut self, set: S) -> EntityId {
        let entity = self.locations.len() as EntityId;

        let archetype_index = match S::get_index(&self.index) {
            Some(index) => index,
            None => {
                let archetype = S::make_archetype();
                let index = self.archetypes.len();

                self.index.insert(S::component_ids(), index);
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

    /// Creates an immutable query into the world. Queries can be used to fetch some specific groups of
    /// components (named archetypes).
    ///
    /// # Example
    ///
    /// ```rust
    /// use tecs::{World, Component, EntityId};
    ///
    /// struct CanFly;
    /// impl Component for CanFly {}
    ///
    /// #[derive(Clone, PartialEq, Debug)]
    /// struct Name(&'static str);
    /// impl Component for Name {}
    ///
    /// enum Color {
    ///     Red,
    ///     Green,
    ///     Blue,
    /// }
    /// impl Component for Color {}
    ///
    /// let mut world = World::new();
    ///
    /// let _entity1 = world.spawn((Name("Sky"), Color::Blue));
    /// let _entity2 = world.spawn((Name("Red Bird"), Color::Red, CanFly));
    /// let _entity3 = world.spawn((Name("Airplane"), CanFly));
    ///
    /// let mut can_fly_names = vec![];
    ///
    /// for (_entity, name, _can_fly) in world.query::<(EntityId, &Name, &CanFly)>() {
    ///     can_fly_names.push(name.clone());
    /// }
    ///
    /// assert_eq!(can_fly_names, [Name("Red Bird"), Name("Airplane")]);
    /// ```
    pub fn query<'w, Q: Query<'w>>(&'w self) -> impl Iterator<Item = Q::Output> + 'w {
        Q::query(self)
    }

    /// Creates a mutable query into the world. Queries can be used to fetch some specific groups of
    /// components (named archetypes) and possibly update them. `query_mut`'s components are
    /// totally modifiable.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tecs::{World, Component, EntityId};
    ///
    /// struct CanFly;
    /// impl Component for CanFly {}
    ///
    /// #[derive(Clone, PartialEq, Debug)]
    /// struct Name(&'static str);
    /// impl Component for Name {}
    ///
    /// enum Color {
    ///     Red,
    ///     Green,
    ///     Blue,
    /// }
    /// impl Component for Color {}
    ///
    /// let mut world = World::new();
    ///
    /// let _entity1 = world.spawn((Name("Sky"), Color::Blue));
    /// let _entity2 = world.spawn((Name("Red Bird"), Color::Red, CanFly));
    /// let _entity3 = world.spawn((Name("Airplane"), CanFly));
    ///
    /// let mut can_fly_names = vec![];
    ///
    /// // you can also opt `EntityId` out if you want
    /// for (name, _can_fly, color) in world.query_mut::<(&Name, &CanFly, &mut Color)>() {
    ///     can_fly_names.push(name.clone());
    ///     // you can modify world through the `query_mut`
    ///     *color = Color::Green;
    /// }
    ///
    /// assert_eq!(can_fly_names, [Name("Red Bird")]);
    /// ```
    pub fn query_mut<'w, Q>(&'w mut self) -> impl Iterator<Item = Q::Output> + 'w
    where
        Q: QueryMut<'w>,
    {
        Q::query_mut(self)
    }

    /// Retrieve a component from a given entity.
    ///
    /// # Note
    ///
    /// There is more optimal way to get components from an entity, see [`World::entity`].
    pub fn get<C: Component>(&self, id: EntityId) -> Option<&C> {
        self.entity(id).get::<C>()
    }

    /// Retrieve a mutable reference to a component from a given entity.
    ///
    /// # Note
    ///
    /// There is more optimal way to get components from an entity, see [`World::entity`].
    pub fn get_mut<C: Component>(&mut self, id: EntityId) -> Option<&mut C> {
        self.entity_mut(id).get::<C>()
    }

    /// Convert lightweight entity id to a stronger handle. Can be used to retrieve components from
    /// an entity efficiently.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tecs::{World, Component};
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Position(f32);
    /// impl Component for Position {}
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Velocity(f32);
    /// impl Component for Velocity {}
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Rotation(f32);
    /// impl Component for Rotation {}
    ///
    /// let mut world = World::new();
    ///
    /// let id = world.spawn((Position(-0.5), Velocity(42.0)));
    /// let entity = world.entity(id);
    ///
    /// assert_eq!(entity.get::<Position>(), Some(&Position(-0.5)));
    /// assert_eq!(entity.get::<Velocity>(), Some(&Velocity(42.0)));
    /// assert_eq!(entity.get::<Rotation>(), None);
    /// ```
    pub fn entity(&self, id: EntityId) -> EntityHandle<'_> {
        let location = self.locations[id as usize];

        EntityHandle {
            id,
            entity_index: location.entity_index,
            archetype: &self.archetypes[location.archetype_index as usize],
        }
    }

    /// Convert lightweight entity id to a stronger mutable handle. Can be used to retrieve and mutate
    /// components from an entity efficiently.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tecs::{World, Component};
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Position(f32);
    /// impl Component for Position {}
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Velocity(f32);
    /// impl Component for Velocity {}
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Rotation(f32);
    /// impl Component for Rotation {}
    ///
    /// let mut world = World::new();
    ///
    /// let id = world.spawn((Position(-0.5), Velocity(42.0)));
    /// let mut entity = world.entity_mut(id);
    ///
    /// *entity.get::<Position>().unwrap() = Position(0.0);
    ///
    /// assert_eq!(entity.get::<Position>(), Some(&mut Position(0.0)));
    /// assert_eq!(entity.get::<Velocity>(), Some(&mut Velocity(42.0)));
    /// assert_eq!(entity.get::<Rotation>(), None);
    /// ```
    pub fn entity_mut(&mut self, id: EntityId) -> EntityHandleMut<'_> {
        let location = self.locations[id as usize];

        EntityHandleMut {
            id,
            entity_index: location.entity_index,
            archetype: &mut self.archetypes[location.archetype_index as usize],
        }
    }
}

/// A strong shared handle to an entity.
#[derive(Clone, Copy)]
pub struct EntityHandle<'w> {
    pub(crate) id: EntityId,
    pub(crate) entity_index: u32,
    pub(crate) archetype: &'w Archetype,
}

impl<'w> EntityHandle<'w> {
    /// Retrieve a component from an entity efficiently.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tecs::{World, Component};
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Position(f32);
    /// impl Component for Position {}
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Velocity(f32);
    /// impl Component for Velocity {}
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Rotation(f32);
    /// impl Component for Rotation {}
    ///
    /// let mut world = World::new();
    ///
    /// let id = world.spawn((Position(-0.5), Velocity(42.0)));
    /// let entity = world.entity(id);
    ///
    /// assert_eq!(entity.get::<Position>(), Some(&Position(-0.5)));
    /// assert_eq!(entity.get::<Velocity>(), Some(&Velocity(42.0)));
    /// assert_eq!(entity.get::<Rotation>(), None);
    /// ```
    pub fn get<C: Component>(&self) -> Option<&'w C> {
        let &component_index = self.archetype.index.get(&TypeId::of::<C>())?;
        let ptr = self.archetype.components[component_index];

        Some(unsafe { ptr.cast::<C>().add(self.entity_index as usize).as_ref()? })
    }

    /// Get the entity id.
    pub fn id(&self) -> EntityId {
        self.id
    }
}

/// A strong mutable handle to an entity.
pub struct EntityHandleMut<'w> {
    pub(crate) id: EntityId,
    pub(crate) entity_index: u32,
    pub(crate) archetype: &'w mut Archetype,
}

impl<'w> EntityHandleMut<'w> {
    /// Retrieve a mutable reference to a component from an entity efficiently.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tecs::{World, Component};
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Position(f32);
    /// impl Component for Position {}
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Velocity(f32);
    /// impl Component for Velocity {}
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Rotation(f32);
    /// impl Component for Rotation {}
    ///
    /// let mut world = World::new();
    ///
    /// let id = world.spawn((Position(-0.5), Velocity(42.0)));
    /// let entity = world.entity(id);
    ///
    /// assert_eq!(entity.get::<Position>(), Some(&Position(-0.5)));
    /// assert_eq!(entity.get::<Velocity>(), Some(&Velocity(42.0)));
    /// assert_eq!(entity.get::<Rotation>(), None);
    /// ```
    pub fn get<C: Component>(&mut self) -> Option<&'w mut C> {
        let &component_index = self.archetype.index.get(&TypeId::of::<C>())?;
        let ptr = self.archetype.components[component_index];

        Some(unsafe { ptr.cast::<C>().add(self.entity_index as usize).as_mut()? })
    }

    /// Get the entity id.
    pub fn id(&self) -> EntityId {
        self.id
    }
}
