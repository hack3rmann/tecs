# `tecs`

## About

Simple implementation of convinient data structure called 'Entity Component System' (or ECS for
short).

## Goals

The goal of this project is to provide simple API with simple implementation.

## Examples

### 1. Spawning an entity and retrieving its components.

```rust
use tecs::{World, Component};

#[derive(Debug, PartialEq)]
struct Name(&'static str);
impl Component for Name {}

#[derive(Debug, PartialEq)]
struct Age(u32);
impl Component for Age {}

let mut world = World::new();

let id = world.spawn((Name("Marcus"), Age(21)));
let entity = world.entity(id);

assert_eq!(entity.get::<Name>(), Some(&Name("Marcus")));
assert_eq!(entity.get::<Age>(), Some(&Age(21)));
```

### 2. Modifying the world through the query.

```rust
use tecs::{World, Component, EntityId};

struct CanFly;
impl Component for CanFly {}

#[derive(Clone, PartialEq, Debug)]
struct Name(&'static str);
impl Component for Name {}

enum Color {
    Red,
    Green,
    Blue,
}
impl Component for Color {}

let mut world = World::new();

let _entity1 = world.spawn((Name("Sky"), Color::Blue));
let _entity2 = world.spawn((Name("Red Bird"), Color::Red, CanFly));
let _entity3 = world.spawn((Name("Airplane"), CanFly));

let mut can_fly_names = vec![];

// you can also opt `EntityId` out if you want
for (name, _can_fly, color) in world.query_mut::<(&Name, &CanFly, &mut Color)>() {
    can_fly_names.push(name.clone());
    // you can modify world through the `query_mut`
    *color = Color::Green;
}

assert_eq!(can_fly_names, [Name("Red Bird")]);
```
