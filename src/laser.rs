use std::borrow::BorrowMut;

use bevy::{pbr::NotShadowCaster, prelude::*};
use bevy_inspector_egui::InspectorOptions;
use rand::random;

use crate::{AnimateTransform, GridLayer, GridMap, GridPosition};

pub struct LaserPlugin;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animate_laser);
        app.add_systems(Update, update_laser);
        app.add_systems(Update, spawn_laser_system);

        app.add_event::<LaserUpdateEvent>();
        app.add_event::<SpawnLaserEvent>();

        app.register_type::<Laser>();
        app.register_type::<Intersection>();
        app.register_type::<IntersectorType>();
    }
}

#[derive(Component, InspectorOptions, Reflect, Debug, Copy, Clone)]
pub struct Laser {
    pub source: Option<Entity>,
    pub from_intersector: Option<Entity>,
    pub to_intersector: Option<Entity>,
    pub index: usize,
    pub direction: Vec2,
    pub start: GridPosition,
    pub end: GridPosition,
}

impl Default for Laser {
    fn default() -> Self {
        Laser {
            source: None,
            from_intersector: None,
            to_intersector: None,
            index: 0,
            direction: Vec2::ZERO,
            start: GridPosition { x: 0, y: 0 },
            end: GridPosition { x: 0, y: 0 },
        }
    }
}

#[derive(Component, Debug, Reflect, InspectorOptions)]
struct Intersection {
    laser_in: Option<Entity>,
    laser_out: Option<Entity>,
}

#[derive(Event, Debug)]
struct SpawnLaserEvent {
    laser_data: Laser,
}

#[derive(Debug)]
pub enum UpdateType {
    Remove,
    Update,
    Place,
}

#[derive(Debug, Component, Copy, Clone, Reflect, PartialEq)]
pub enum IntersectorType {
    Emitter,
    Reflector,
}

#[derive(Event, Debug)]
pub struct LaserUpdateEvent {
    pub entity: Entity,
    pub update_type: UpdateType,
    pub intersector: IntersectorType,
    pub grid_position: GridPosition,
}

fn update_laser(
    mut commands: Commands,
    mut events: EventReader<LaserUpdateEvent>,
    mut grid: ResMut<GridMap>,
    q_laser: Query<(Entity, &Laser)>,
    mut ev_spawn_laser: EventWriter<SpawnLaserEvent>,
    mut q_intersector_types: Query<&IntersectorType, With<Intersection>>,
) {
    for ev in events.read() {
        println!("LaserUpdateEvent: {:?}", ev);
        match ev.update_type {
            UpdateType::Place => {
                // Place
                match ev.intersector {
                    IntersectorType::Emitter => {
                        // Emitter
                        walk_laser(
                            0,
                            ev.entity,
                            ev.grid_position,
                            Vec2::new(0.0, 1.0),
                            grid.borrow_mut(),
                            ev_spawn_laser.borrow_mut(),
                            q_intersector_types.borrow_mut(),
                        );
                    }
                    IntersectorType::Reflector => {
                        // Reflector
                    }
                }
            }
            UpdateType::Update => {
                // Update
            }
            UpdateType::Remove => {
                // Remove
                match ev.intersector {
                    IntersectorType::Emitter => {
                        // Emitter
                        for (laser_entity, laser) in q_laser.iter() {
                            if laser.source == Some(ev.entity) {
                                commands.entity(laser_entity).despawn_recursive();
                            }
                        }
                    }
                    IntersectorType::Reflector => {
                        // Reflector
                    }
                }
            }
        }
    }
}

// recursive function to walk the laser with inputs (grid_pos, direction)
// if the next grid_pos is a building, spawn laser with appropriate length
fn walk_laser(
    index: usize,
    source: Entity,
    grid_pos: GridPosition,
    direction: Vec2,
    grid: &mut ResMut<GridMap>,
    ev: &mut EventWriter<SpawnLaserEvent>,
    q_intersector_types: &mut Query<&IntersectorType, With<Intersection>>,
) {
    let start_pos = GridPosition {
        x: (grid_pos.x - index as i32 * direction.x as i32),
        y: (grid_pos.y - index as i32 * direction.y as i32),
    };

    let next_index = index + 1;
    let next_grid_pos = GridPosition {
        x: grid_pos.x + direction.x as i32,
        y: grid_pos.y + direction.y as i32,
    };

    if index > 10 {
        println!("Max index reached");
        ev.send(SpawnLaserEvent {
            laser_data: Laser {
                source: Some(source),
                from_intersector: Some(source),
                to_intersector: None,
                index: 0,
                direction,
                start: start_pos,
                end: next_grid_pos,
            },
        });
        return;
    }

    // check if next grid_pos is a building
    if grid.contains(GridLayer::Build, next_grid_pos) {
        let entity = grid.get(GridLayer::Build, next_grid_pos).unwrap();
        println!("Building at {:?}", next_grid_pos);
        ev.send(SpawnLaserEvent {
            laser_data: Laser {
                source: Some(source),
                from_intersector: Some(source),
                to_intersector: Some(*entity),
                index: 0,
                direction,
                start: start_pos,
                end: next_grid_pos,
            },
        });

        // if this building is a reflector, walk the laser again but with the new direction based on the reflector
        if let Ok(intersector) = q_intersector_types.get(*entity) {
            if *intersector == IntersectorType::Reflector {
                walk_laser(
                    0,
                    *entity,
                    next_grid_pos,
                    direction,
                    grid,
                    ev,
                    q_intersector_types,
                );
            }
        }
        return;
    };
    walk_laser(
        next_index,
        source,
        next_grid_pos,
        direction,
        grid,
        ev,
        q_intersector_types,
    );
}

fn spawn_laser_system(
    mut commands: Commands,
    mut ev_spawn_laser: EventReader<SpawnLaserEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    intersector_query: Query<&IntersectorType, With<Intersection>>,
) {
    for ev in ev_spawn_laser.read() {
        println!("SpawnLaserEvent recieved: {:?}", ev);
        let laser = spawn_laser(&mut commands, ev.laser_data, &mut meshes, &mut materials);
        // add intersection component to previous building
        if let Some(from_intersector) = ev.laser_data.from_intersector {
            commands.entity(from_intersector).insert(Intersection {
                laser_in: None,
                laser_out: Some(laser),
            });
        }

        // add intersection component to intersecting building
        if let Some(to_intersector) = ev.laser_data.to_intersector {
            commands.entity(to_intersector).insert(Intersection {
                laser_in: Some(laser),
                laser_out: None,
            });
        };

        // add laser component to laser entity
        commands.entity(laser).insert(ev.laser_data);
    }
}

fn animate_laser(time: Res<Time>, mut query: Query<&mut Transform, With<Laser>>) {
    for mut transform in &mut query {
        transform.rotation *= Quat::from_rotation_y(time.delta_seconds() * 5. * random::<f32>());
    }
}

fn spawn_laser(
    commands: &mut Commands,
    laser: Laser,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    let laser_length =
        ((laser.start.x - laser.end.x) as f32 + (laser.start.y - laser.end.y) as f32).abs();
    println!("Laser length: {}", laser_length);
    let position = Vec2::new(laser.end.x as f32 / 2.0, laser.end.y as f32 / 2.0);
    println!(
        "Position: {:?}, GridFrom: {:?}, GridTo: {:?}",
        position, laser.start, laser.end
    );

    let laser = (
        PbrBundle {
            mesh: meshes.add(Cylinder::new(0.5, 1.0).mesh().resolution(50)),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                reflectance: 0.5,
                diffuse_transmission: 0.5,
                specular_transmission: 1.0,
                perceptual_roughness: 0.5,
                thickness: 4.0,
                ior: 1.18,
                emissive: Color::ORANGE_RED * 40.0,
                ..default()
            }),
            transform: Transform::from_translation(Vec3::new(0.0, 0.25, 0.0))
                .with_scale(Vec3::new(0.04, 0.0, 0.06))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            ..Default::default()
        },
        AnimateTransform {
            target_scale: Vec3::new(0.03, laser_length - 0.5, 0.04),
            target_position: Vec3::new(position.x, 0.5, position.y),
            duration: 2.5,
            ..Default::default()
        },
        Laser::default(),
        NotShadowCaster,
        Name::new("Laser"),
    );
    commands.spawn(laser).id()
}
