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
        app.add_systems(Update, intersection_system);

        app.add_event::<LaserUpdateEvent>();

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
    source: Option<Entity>,
    laser_in: Option<Entity>,
    laser_out: Option<Entity>,
    laser_out_direction: Vec2,
    from: Option<Entity>,
    to: Option<Entity>,
}

impl Default for Intersection {
    fn default() -> Self {
        Intersection {
            source: None,
            laser_in: None,
            laser_out: None,
            laser_out_direction: Vec2::ZERO,
            from: None,
            to: None,
        }
    }
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

#[derive(Component)]
struct UpdatePending;

fn intersection_system(
    mut commands: Commands,
    grid: Res<GridMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut set: ParamSet<(
        Query<(Entity, &mut Intersection, &IntersectorType), With<UpdatePending>>,
        Query<&mut Intersection>,
    )>,
) {
    let mut laser_entity: Option<Entity> = None;
    let mut source: Option<Entity> = None;
    let mut from_intersector: Option<Entity> = None;
    let mut to_intersector: Option<Entity> = None;
    for (intersection_entity, mut intersection, intersection_type) in set.p0().iter_mut() {
        // remove update pending
        commands
            .entity(intersection_entity)
            .remove::<UpdatePending>();
        println!("Update intersection: {:?}", intersection_entity);

        // remove all connected lasers
        // if let Some(laser_in) = intersection.laser_in {
        //     println!("Remove laser in: {:?}", laser_in);
        //     commands.entity(laser_in).despawn_recursive();
        // }
        //
        // if let Some(laser_out) = intersection.laser_out {
        //     println!("Remove laser out: {:?}", laser_out);
        //     commands.entity(laser_out).despawn_recursive();
        // }

        if let Some(to_intersector_entity) = intersection.to {
            to_intersector = Some(to_intersector_entity);
        }

        match intersection_type {
            IntersectorType::Emitter => {
                source = Some(intersection_entity);
                from_intersector = Some(intersection_entity);
            }
            IntersectorType::Reflector => {
                if intersection.laser_in.is_none() {
                    return;
                }
            }
        }

        // spawn laser (check for collisions in laser_direction)
        let max_length = 10;
        let start_pos = GridPosition {
            x: intersection.laser_out_direction.x as i32,
            y: intersection.laser_out_direction.y as i32,
        };
        let mut end_pos = GridPosition {
            x: intersection.laser_out_direction.x as i32 * max_length,
            y: intersection.laser_out_direction.y as i32 * max_length,
        };

        for i in 1..max_length {
            let next_grid_pos = GridPosition {
                x: intersection.laser_out_direction.x as i32 * i,
                y: intersection.laser_out_direction.y as i32 * i,
            };
            if grid.contains(GridLayer::Build, next_grid_pos) {
                end_pos = next_grid_pos;
                // insert update pending
                let entity = grid.get(GridLayer::Build, next_grid_pos).unwrap();
                to_intersector = Some(*entity);
            }
        }

        let laser = spawn_laser(
            &mut commands,
            Laser {
                source,
                from_intersector,
                to_intersector,
                index: 0,
                direction: intersection.laser_out_direction,
                start: start_pos,
                end: end_pos,
            },
            meshes.borrow_mut(),
            materials.borrow_mut(),
        );
        laser_entity = Some(laser);

        // insert data to laser

        // insert intersectiondata to current intersection
        intersection.source = source;
        intersection.laser_out = Some(laser);
    }

    if let Some(child_entity) = to_intersector {
        // insert update pending on child entity
        println!("Insert update pending on child entity: {:?}", child_entity);
        commands.entity(child_entity).insert(UpdatePending);

        // get child intersection
        let mut set_p1 = set.p1();
        let mut to_intersection = set_p1.get_mut(child_entity).unwrap();

        // set laser_in to spawned laser
        to_intersection.source = source;
        to_intersection.laser_in = laser_entity;
        to_intersection.from = from_intersector;
        to_intersection.to = to_intersector;
    }
}

fn update_laser(
    mut commands: Commands,
    mut events: EventReader<LaserUpdateEvent>,
    q_laser: Query<(Entity, &Laser)>,
) {
    for ev in events.read() {
        println!("LaserUpdateEvent: {:?}", ev);
        match ev.update_type {
            UpdateType::Place => {
                // Place
                match ev.intersector {
                    IntersectorType::Emitter => {
                        // Emitter
                        commands.entity(ev.entity).insert((
                            Intersection {
                                laser_out_direction: Vec2::new(0.0, 1.0),
                                ..default()
                            },
                            UpdatePending,
                        ));
                    }
                    IntersectorType::Reflector => {
                        // Reflector
                        commands.entity(ev.entity).insert((
                            Intersection {
                                laser_out_direction: Vec2::new(1.0, 0.0),
                                ..default()
                            },
                            UpdatePending,
                        ));
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
                            println!("Laser: {:?}, source: {:?}", laser_entity, laser.source);
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

    let laser_entity = (
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
        Laser { ..laser },
        NotShadowCaster,
        Name::new("Laser"),
    );
    commands.spawn(laser_entity).id()
}
