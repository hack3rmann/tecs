#![allow(unused)]

#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct Velocity;

#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct Position;

type Entity = u32;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct VelocityArchetype {
    pub components: Vec<Velocity>,
    pub entities: Vec<Entity>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct PositionArchetype {
    pub components: Vec<Position>,
    pub entities: Vec<Entity>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct PositionVelocityArchetype {
    pub positions: Vec<Position>,
    pub velocities: Vec<Velocity>,
    pub entities: Vec<Entity>,
}

#[derive(Clone, Debug, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
pub enum EntityArchetype {
    Velocity,
    Position,
    PositionVelocity,
}

#[derive(Clone, Debug, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct Location {
    pub archetype: EntityArchetype,
    pub index: usize,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct World {
    pub velocity_archetype: VelocityArchetype,
    pub position_archetype: PositionArchetype,
    pub position_velocity_archetype: PositionVelocityArchetype,
    pub locations: Vec<Location>,
}

impl World {
    pub fn spawn_with_position(&mut self, value: Position) -> Entity {
        let entity = self.locations.len() as Entity;
        let location = Location {
            archetype: EntityArchetype::Position,
            index: self.position_archetype.components.len(),
        };

        self.locations.push(location);
        self.position_archetype.entities.push(entity);
        self.position_archetype.components.push(value);

        entity
    }

    pub fn spawn_with_velocity(&mut self, value: Velocity) -> Entity {
        let entity = self.locations.len() as Entity;
        let location = Location {
            archetype: EntityArchetype::Velocity,
            index: self.velocity_archetype.components.len(),
        };

        self.locations.push(location);
        self.velocity_archetype.entities.push(entity);
        self.velocity_archetype.components.push(value);

        entity
    }

    pub fn spawn_with_position_and_velocity(
        &mut self,
        position: Position,
        velocity: Velocity,
    ) -> Entity {
        let entity = self.locations.len() as Entity;
        let location = Location {
            archetype: EntityArchetype::PositionVelocity,
            index: self.position_velocity_archetype.positions.len(),
        };

        self.locations.push(location);
        self.position_velocity_archetype.entities.push(entity);
        self.position_velocity_archetype.positions.push(position);
        self.position_velocity_archetype.velocities.push(velocity);

        entity
    }

    pub fn velocities(&mut self) -> impl Iterator<Item = (Entity, &mut Velocity)> {
        Iterator::chain(
            self.velocity_archetype
                .entities
                .iter()
                .copied()
                .zip(self.velocity_archetype.components.iter_mut()),
            self.position_velocity_archetype
                .entities
                .iter()
                .copied()
                .zip(self.position_velocity_archetype.velocities.iter_mut()),
        )
    }

    pub fn positions(&mut self) -> impl Iterator<Item = (Entity, &mut Position)> {
        Iterator::chain(
            self.position_archetype
                .entities
                .iter()
                .copied()
                .zip(self.position_archetype.components.iter_mut()),
            self.position_velocity_archetype
                .entities
                .iter()
                .copied()
                .zip(self.position_velocity_archetype.positions.iter_mut()),
        )
    }

    pub fn positions_and_velocities(
        &mut self,
    ) -> impl Iterator<Item = (Entity, &mut Position, &mut Velocity)> {
        self.position_velocity_archetype
            .entities
            .iter()
            .copied()
            .zip(self.position_velocity_archetype.positions.iter_mut())
            .zip(self.position_velocity_archetype.velocities.iter_mut())
            .map(|((x, y), z)| (x, y, z))
    }
}
