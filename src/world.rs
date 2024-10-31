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
        self.entity(id).get::<C>()
    }

    pub fn get_mut<C: Component>(&mut self, id: EntityId) -> Option<&mut C> {
        self.entity_mut(id).get::<C>()
    }

    pub fn entity(&self, id: EntityId) -> EntityHandle<'_> {
        let location = self.locations[id as usize];

        EntityHandle {
            id,
            entity_index: location.entity_index,
            archetype: &self.archetypes[location.archetype_index as usize],
        }
    }

    pub fn entity_mut(&mut self, id: EntityId) -> EntityHandleMut<'_> {
        let location = self.locations[id as usize];

        EntityHandleMut {
            id,
            entity_index: location.entity_index,
            archetype: &mut self.archetypes[location.archetype_index as usize],
        }
    }
}

#[derive(Clone, Copy)]
pub struct EntityHandle<'w> {
    pub(crate) id: EntityId,
    pub(crate) entity_index: u32,
    pub(crate) archetype: &'w Archetype,
}

impl<'w> EntityHandle<'w> {
    pub fn get<C: Component>(&self) -> Option<&'w C> {
        let &component_index = self.archetype.index.get(&TypeId::of::<C>())?;
        let ptr = self.archetype.components[component_index];

        Some(unsafe { ptr.cast::<C>().add(self.entity_index as usize).as_ref()? })
    }
}

pub struct EntityHandleMut<'w> {
    pub(crate) id: EntityId,
    pub(crate) entity_index: u32,
    pub(crate) archetype: &'w mut Archetype,
}

impl<'w> EntityHandleMut<'w> {
    pub fn get<C: Component>(&mut self) -> Option<&'w mut C> {
        let &component_index = self.archetype.index.get(&TypeId::of::<C>())?;
        let ptr = self.archetype.components[component_index];

        Some(unsafe { ptr.cast::<C>().add(self.entity_index as usize).as_mut()? })
    }
}
