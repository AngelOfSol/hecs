// Copyright 2019 Google LLC
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::{any::TypeId, collections::HashMap};

use bencher::{benchmark_group, benchmark_main, Bencher};
use hecs::*;

#[derive(Clone)]
struct Position(f32);
#[derive(Clone)]
struct Velocity(f32);

fn spawn_tuple(b: &mut Bencher) {
    let mut world = World::new();
    b.iter(|| {
        world.spawn((Position(0.0), Velocity(0.0)));
    });
}

fn spawn_static(b: &mut Bencher) {
    #[derive(Bundle)]
    struct Bundle {
        pos: Position,
        vel: Velocity,
    };

    let mut world = World::new();
    b.iter(|| {
        world.spawn(Bundle {
            pos: Position(0.0),
            vel: Velocity(0.0),
        });
    });
}

fn spawn_batch(b: &mut Bencher) {
    #[derive(Bundle)]
    struct Bundle {
        pos: Position,
        vel: Velocity,
    };

    let mut world = World::new();
    b.iter(|| {
        world
            .spawn_batch((0..1_000).map(|_| Bundle {
                pos: Position(0.0),
                vel: Velocity(0.0),
            }))
            .for_each(|_| {});
        world.clear();
    });
}

fn iterate_100k(b: &mut Bencher) {
    let mut world = World::new();
    for i in 0..100_000 {
        world.spawn((Position(-(i as f32)), Velocity(i as f32)));
    }
    b.iter(|| {
        for (_, (pos, vel)) in &mut world.query::<(&mut Position, &Velocity)>() {
            pos.0 += vel.0;
        }
    })
}

const CLONE_COUNT: u32 = 100_000;

fn clone_100k(b: &mut Bencher) {
    let mut world = World::new();
    for i in 0..CLONE_COUNT {
        world.spawn((
            Position(-(i as f32)),
            Velocity(i as f32),
            String::from("test"),
        ));
    }
    b.iter(|| {
        let _ = world.clone();
    });
}
fn spawn_at_clone_100k(b: &mut Bencher) {
    let mut world = World::new();
    for i in 0..CLONE_COUNT {
        world.spawn((
            Position(-(i as f32)),
            Velocity(i as f32),
            String::from("test"),
        ));
    }
    b.iter(|| {
        let mut cloned_world = World::new();
        cloned_world.reserve::<(Position, Velocity, String)>(CLONE_COUNT);
        for (entity, (lhs, rhs, ths)) in world.query::<(&Position, &Velocity, &String)>().iter() {
            cloned_world.spawn_at(entity, (lhs.clone(), rhs.clone(), ths.clone()));
        }
    });
}
fn spawn_at_clone_index_100k(b: &mut Bencher) {
    let mut world = World::new();
    for i in 0..CLONE_COUNT {
        world.spawn((
            Position(-(i as f32)),
            Velocity(i as f32),
            String::from("test"),
        ));
    }
    fn clone_on_to<T: Clone + hecs::Component>(x: EntityRef<'_>, builder: &mut EntityBuilder) {
        if let Some(value) = x.get::<T>() {
            let value = (*value).clone();
            builder.add(value);
        }
    }
    const FUNCTIONS: &[&dyn Fn(EntityRef<'_>, &mut EntityBuilder)] = &[
        &clone_on_to::<Position>,
        &clone_on_to::<Velocity>,
        &clone_on_to::<String>,
    ];

    b.iter(|| {
        let mut cloned_world = World::new();
        for (id, entity_ref) in world.iter() {
            let mut builder = EntityBuilder::new();
            cloned_world.reserve::<(Position, Velocity, String)>(CLONE_COUNT);
            for func in FUNCTIONS {
                (**func)(entity_ref, &mut builder);
            }
            cloned_world.spawn_at(id, builder.build());
        }
    });
}

fn build(b: &mut Bencher) {
    let mut world = World::new();
    let mut builder = EntityBuilder::new();
    b.iter(|| {
        builder.add(Position(0.0)).add(Velocity(0.0));
        world.spawn(builder.build());
    });
}

benchmark_group!(
    benches,
    spawn_tuple,
    spawn_static,
    spawn_batch,
    iterate_100k,
    clone_100k,
    spawn_at_clone_100k,
    spawn_at_clone_index_100k,
    build,
);
benchmark_main!(benches);
