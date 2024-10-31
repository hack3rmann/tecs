use crate::{
    query::{Query, QueryMut},
    Archetype, ComponentSet, EntityId, Location,
};
use std::{any::TypeId, collections::HashMap};

#[derive(Default)]
pub struct World {
    pub(crate) archetypes: Vec<Archetype>,
    pub(crate) locations: Vec<Location>,
    pub(crate) index: HashMap<Box<[TypeId]>, usize>,
}

pub trait Component: Sized + 'static {}

impl World {
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

    pub fn query<'w, Q: Query<'w>>(&'w mut self) -> impl Iterator<Item = Q::Output> + 'w {
        Q::query(self)
    }

    pub fn query_mut<'w, Q>(&'w mut self) -> impl Iterator<Item = Q::Output> + 'w
    where
        Q: QueryMut<'w>,
    {
        Q::query_mut(self)
    }

    pub fn get<C: Component>(&self, id: EntityId) -> Option<&C> {
        let location = self.locations[id as usize];
        let archetype = &self.archetypes[location.archetype_index as usize];
        let &component_index = archetype.index.get(&TypeId::of::<C>())?;
        let ptr = archetype.components[component_index];

        Some(unsafe {
            ptr.cast::<C>()
                .add(location.entity_index as usize)
                .as_ref()?
        })
    }

    pub fn get_mut<C: Component>(&mut self, id: EntityId) -> Option<&mut C> {
        let location = self.locations[id as usize];
        let archetype = &self.archetypes[location.archetype_index as usize];
        let &component_index = archetype.index.get(&TypeId::of::<C>())?;
        let ptr = archetype.components[component_index];

        Some(unsafe {
            ptr.cast::<C>()
                .add(location.entity_index as usize)
                .as_mut()?
        })
    }

    pub fn entity(&self, id: EntityId) -> EntityHandle<'_> {
        EntityHandle { id, world: self }
    }

    pub fn entity_mut(&mut self, id: EntityId) -> EntityHandleMut<'_> {
        EntityHandleMut { id, world: self }
    }
}

#[derive(Clone, Copy)]
pub struct EntityHandle<'w> {
    pub(crate) id: EntityId,
    pub(crate) world: &'w World,
}

impl EntityHandle<'_> {
    pub fn get<C: Component>(&self) -> Option<&C> {
        self.world.get::<C>(self.id)
    }
}

pub struct EntityHandleMut<'w> {
    pub(crate) id: EntityId,
    pub(crate) world: &'w mut World,
}

impl EntityHandleMut<'_> {
    pub fn get_mut<C: Component>(&mut self) -> Option<&mut C> {
        self.world.get_mut::<C>(self.id)
    }
}
