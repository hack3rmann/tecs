use crate::{
    query::{Query, QueryMut},
    Archetype, ComponentSet, Entity, Location,
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
    pub fn spawn<S: ComponentSet>(&mut self, set: S) -> Entity {
        let entity = self.locations.len() as Entity;

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
}
