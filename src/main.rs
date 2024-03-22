#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use bevy::utils::HashMap;
use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    pbr::PointLightShadowMap,
    prelude::*,
    window::{PresentMode, PrimaryWindow},
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_inspector_egui::InspectorOptions;
use camera::{CameraPlugin, MainCamera};
use fps::FPSPlugin;
use laser::*;
use std::borrow::BorrowMut;

mod camera;
mod fps;
mod laser;

fn main() {
    let mut app = App::new();

    // plugins
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "spectrum".into(),
            present_mode: PresentMode::AutoNoVsync,
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
    }))
    .add_plugins(WorldInspectorPlugin::new())
    .add_plugins((FPSPlugin, FrameTimeDiagnosticsPlugin))
    .add_plugins(CameraPlugin)
    .add_plugins(LaserPlugin);

    // resources
    app.insert_resource(ClearColor(Color::BLACK))
        .insert_resource(PointLightShadowMap { size: 2048 })
        .insert_resource(Game::default())
        .insert_resource(AmbientLight {
            brightness: 0.0,
            ..default()
        })
        .insert_resource(Msaa::default())
        .insert_resource(MouseWorldPosition(Vec2::ZERO))
        .insert_resource(MouseGridPosition(Vec2::ZERO))
        .insert_resource(GridMap::default());
    app.insert_state(AppState::InGame);

    // events
    app.add_event::<AnimationCompleteEvent>();

    // systems
    app.add_systems(Startup, setup);
    app.add_systems(OnEnter(AppState::InGame), spawn_color_wells);
    app.add_systems(
        Update,
        (
            cursor_system,
            move_cursor_attachment,
            place_block,
            animate_transform_system,
            destroy_block_system,
            on_building_destroy,
            update_current_placeable,
        )
            .run_if(in_state(AppState::InGame)),
    );

    // types
    app.register_type::<GridPosition>();

    app.run();
}

const GRID_SCALE: f32 = 1.0;

// define the game state
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    InGame,
}

#[derive(Component, Copy, Clone, Eq, PartialEq, Hash, Reflect, InspectorOptions, Debug)]
pub struct GridPosition {
    x: i32,
    y: i32,
}

impl From<Vec2> for GridPosition {
    fn from(v: Vec2) -> Self {
        Self {
            x: v.x as i32,
            y: v.y as i32,
        }
    }
}

#[derive(Component, PartialEq, Clone, Debug)]
enum Placeable {
    Collector,
    Mirror,
}

#[derive(Resource, Default)]
struct Game {
    current_placeable: Option<Placeable>,
}

#[derive(Event)]
struct AnimationCompleteEvent(Entity);

#[derive(Component)]
struct DeletionPending;

#[derive(Component)]
struct Active;

#[derive(Component)]
struct Building;

#[derive(Component)]
struct Collector;

#[derive(Component)]
struct BuildBlock;

#[derive(Component)]
struct AnimateTransform {
    target_position: Vec3,
    target_scale: Vec3,
    duration: f32,
    elapsed: f32,
}

impl Default for AnimateTransform {
    fn default() -> Self {
        Self {
            target_scale: Vec3::splat(1.0),
            target_position: Vec3::ZERO,
            duration: 0.5,
            elapsed: 0.0,
        }
    }
}

#[derive(Component)]
struct CursorAttachment;

#[derive(Resource, Default)]
struct MouseWorldPosition(Vec2);

#[derive(Resource, Default)]
struct MouseGridPosition(Vec2);

#[derive(Component)]
struct ColorWell {
    color: Color,
}

impl Default for ColorWell {
    fn default() -> Self {
        Self {
            color: Color::ORANGE_RED,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
enum GridLayer {
    Ground,
    Build,
    Laser,
}
#[derive(Resource, Default)]
struct GridMap {
    map: HashMap<(GridLayer, GridPosition), Entity>,
}

impl GridMap {
    fn get(&self, layer: GridLayer, position: GridPosition) -> Option<&Entity> {
        self.map.get(&(layer, position))
    }

    fn set(&mut self, layer: GridLayer, position: GridPosition, value: Entity) -> Result<(), ()> {
        if self.map.contains_key(&(layer, position)) {
            return Err(());
        }

        self.map.insert((layer, position), value);
        Ok(())
    }

    fn remove(&mut self, layer: GridLayer, position: GridPosition) -> Result<(), ()> {
        if !self.map.contains_key(&(layer, position)) {
            return Err(());
        }

        self.map.remove(&(layer, position));
        Ok(())
    }

    fn contains(&self, layer: GridLayer, position: GridPosition) -> bool {
        self.map.contains_key(&(layer, position))
    }
}

fn spawn_color_wells(
    mut grid_map: ResMut<GridMap>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let position = Vec3::new(0.0 * GRID_SCALE, -0.49, 0.0 * GRID_SCALE);
    let color = Color::ORANGE_RED;
    let e = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(StandardMaterial {
                base_color: color,
                reflectance: 0.5,
                emissive: color * 10.0,
                ..default()
            }),
            transform: Transform::from_translation(position),
            ..Default::default()
        },
        GridPosition { x: 0, y: 0 },
        ColorWell { color },
        Name::new("Color Well"),
    ));
    grid_map
        .set(GridLayer::Ground, GridPosition { x: 0, y: 0 }, e.id())
        .unwrap();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let plane_mesh = meshes.add(Plane3d::default().mesh().size(100.0, 100.0));

    // Directional light
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                shadows_enabled: true,
                illuminance: 300.0,
                ..default()
            },
            transform: Transform::from_xyz(10.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        Name::new("Directional Light"),
    ));

    // Floor
    let floor_mat = materials.add(StandardMaterial {
        base_color: Color::rgb(0.03, 0.03, 0.03),
        reflectance: 0.0,
        perceptual_roughness: 1.0,
        ..default()
    });

    commands.spawn((
        PbrBundle {
            mesh: plane_mesh.clone(),
            material: floor_mat.clone(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        Name::new("Floor"),
    ));

    // spawn cursor block
    let cursor_size = 0.9;
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(cursor_size, 0.1, cursor_size)),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                reflectance: 0.5,
                diffuse_transmission: 0.5,
                specular_transmission: 0.5,
                perceptual_roughness: 0.5,
                thickness: 0.2,
                ..default()
            }),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        },
        Name::new("Cursor Block"),
        CursorAttachment,
        Active,
    ));
}

fn update_current_placeable(mut game: ResMut<Game>, inputs: Res<ButtonInput<KeyCode>>) {
    if inputs.just_pressed(KeyCode::Digit1) {
        println!("Collector selected");
        game.current_placeable = Some(Placeable::Collector);
    }

    if inputs.just_pressed(KeyCode::Digit2) {
        println!("Mirror selected");
        game.current_placeable = Some(Placeable::Mirror);
    }

    if inputs.just_pressed(KeyCode::Escape) {
        println!("Deselected");
        game.current_placeable = None;
    }
}

fn cursor_system(
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut mouse_world_position: ResMut<MouseWorldPosition>,
    mut mouse_grid_position: ResMut<MouseGridPosition>,
) {
    let (camera, camera_transform) = q_camera.single();
    let window = q_window.single();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| {
            // get the intersection of the ray with the xz plane on y=0
            // t is the distance from the ray origin to the intersection point
            let t = -ray.origin.y / ray.direction.y;
            Vec3::new(
                ray.origin.x + t * ray.direction.x,
                0.0,
                ray.origin.z + t * ray.direction.z,
            )
        })
    {
        mouse_world_position.0 = Vec2::new(world_position.x, world_position.z);
        mouse_grid_position.0 = world_to_grid(Vec2::new(world_position.x, world_position.z));
    }
}

fn spawn_cursor_attachments(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.9, 1.0, 0.9)),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                reflectance: 0.5,
                diffuse_transmission: 0.5,
                specular_transmission: 1.0,
                perceptual_roughness: 0.5,
                thickness: 4.0,
                ior: 1.18,
                ..default()
            }),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        },
        Placeable::Collector,
    ));
}

fn move_cursor_attachment(
    time: Res<Time>,
    mut cursor_attachement: Query<&mut Transform, With<CursorAttachment>>,
    mouse_grid_pos: Res<MouseGridPosition>,
) {
    for mut transform in cursor_attachement.iter_mut() {
        transform.translation = transform.translation.lerp(
            Vec3::new(
                mouse_grid_pos.0.x,
                transform.translation.y,
                mouse_grid_pos.0.y,
            ),
            time.delta_seconds() * 15.,
        );
    }
}

fn update_cursor_attachment(
    game: Res<Game>,
    mut commands: Commands,
    cursor_attachements: Query<(Entity, &Placeable), With<CursorAttachment>>,
    active_cursor_attachement: Query<(Entity), (With<CursorAttachment>, With<Active>)>,
) {
    let active_entity = match active_cursor_attachement.get_single() {
        Ok(entity) => entity,
        Err(_) => return,
    };

    if let Some(current_placeable) = &game.current_placeable {
        match current_placeable {
            Placeable::Collector => {
                // change cursor attachment to collector
            }
            Placeable::Mirror => {
                // change cursor attachment to mirror
                // commands.spawn(glasscube).insert(Active);
            }
        }
    }
}

fn world_to_grid(world_position: Vec2) -> Vec2 {
    Vec2::new(
        (world_position.x / GRID_SCALE).round(),
        (world_position.y / GRID_SCALE).round(),
    )
}

fn grid_to_world(grid_position: &GridPosition) -> Vec2 {
    Vec2::new(
        grid_position.x as f32 * GRID_SCALE,
        grid_position.y as f32 * GRID_SCALE,
    )
}

fn place_block(
    mut commands: Commands,
    mut grid_map: ResMut<GridMap>,
    game: Res<Game>,
    mouse_grid_pos: Res<MouseGridPosition>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    inactive_color_wells: Query<
        (Entity, &GridPosition, &mut Handle<StandardMaterial>),
        With<ColorWell>,
    >,
    mut ev_laser_update: EventWriter<LaserUpdateEvent>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let grid_pos = GridPosition::from(mouse_grid_pos.0);
        let grid_map_entity = grid_map.get(GridLayer::Ground, grid_pos);
        match &game.current_placeable {
            Some(Placeable::Collector) => {
                println!("Attempting to place collector");
                // place collector
                if let Some(grid_map_entity) = grid_map_entity {
                    if let Ok(inactive_color_well) = inactive_color_wells.get(*grid_map_entity) {
                        commands.entity(inactive_color_well.0).insert(Active);

                        if grid_map.contains(GridLayer::Build, GridPosition::from(mouse_grid_pos.0))
                        {
                            return;
                        }

                        println!("Color well found");
                        let collector = spawn_collector(
                            commands.borrow_mut(),
                            Vec3::new(
                                grid_pos.x as f32 * GRID_SCALE,
                                0.5,
                                grid_pos.y as f32 * GRID_SCALE,
                            ),
                            grid_pos,
                            meshes.borrow_mut(),
                            materials.borrow_mut(),
                        );
                        ev_laser_update.send(LaserUpdateEvent {
                            entity: collector,
                            update_type: UpdateType::Place,
                            intersector: IntersectorType::Emitter,
                            grid_position: GridPosition::from(mouse_grid_pos.0),
                        });
                        grid_map
                            .set(
                                GridLayer::Build,
                                GridPosition::from(mouse_grid_pos.0),
                                collector,
                            )
                            .unwrap();
                    }
                }
            }
            Some(Placeable::Mirror) => {
                // place mirror
                if grid_map_entity.is_none() {
                    if grid_map.contains(GridLayer::Build, GridPosition::from(mouse_grid_pos.0)) {
                        return;
                    }
                    let mirror = spawn_mirror(
                        commands.borrow_mut(),
                        Vec3::new(
                            grid_pos.x as f32 * GRID_SCALE,
                            0.5,
                            grid_pos.y as f32 * GRID_SCALE,
                        ),
                        grid_pos,
                        meshes.borrow_mut(),
                        materials.borrow_mut(),
                    );

                    ev_laser_update.send(LaserUpdateEvent {
                        entity: mirror,
                        update_type: UpdateType::Place,
                        intersector: IntersectorType::Reflector,
                        grid_position: GridPosition::from(mouse_grid_pos.0),
                    });
                    grid_map
                        .set(
                            GridLayer::Build,
                            GridPosition::from(mouse_grid_pos.0),
                            mirror,
                        )
                        .unwrap();
                }
            }
            None => {}
        }
    }
}
#[derive(Component)]
struct Mirror;
fn spawn_mirror(
    commands: &mut Commands,
    position: Vec3,
    grid_pos: GridPosition,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    let mirror = (
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.05, 0.8, 0.8)),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                reflectance: 1.0,
                diffuse_transmission: 0.2,
                specular_transmission: 0.3,
                perceptual_roughness: 0.0,
                thickness: 4.0,
                ior: 1.18,
                ..default()
            }),
            transform: Transform::from_translation(Vec3::new(position.x, -0.4, position.z))
                .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_4)),
            ..Default::default()
        },
        AnimateTransform {
            target_position: Vec3::new(grid_to_world(&grid_pos).x, 0.5, grid_to_world(&grid_pos).y),
            target_scale: Vec3::splat(1.0),
            duration: 1.5,
            ..default()
        },
        GridPosition {
            x: grid_pos.x,
            y: grid_pos.y,
        },
        Building,
        Mirror,
        IntersectorType::Reflector,
        Name::new("Mirror"),
    );

    commands.spawn(mirror).id()
}

fn spawn_collector(
    commands: &mut Commands,
    position: Vec3,
    grid_pos: GridPosition,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    let collector = (
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.95, 1.0, 0.95)),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                reflectance: 0.5,
                diffuse_transmission: 0.5,
                specular_transmission: 1.0,
                perceptual_roughness: 0.5,
                thickness: 4.0,
                ior: 1.18,
                ..default()
            }),
            transform: Transform::from_translation(Vec3::new(position.x, -0.4, position.z)),
            ..Default::default()
        },
        AnimateTransform {
            target_position: Vec3::new(grid_to_world(&grid_pos).x, 0.5, grid_to_world(&grid_pos).y),
            target_scale: Vec3::splat(1.0),
            duration: 1.5,
            ..default()
        },
        GridPosition {
            x: grid_pos.x,
            y: grid_pos.y,
        },
        Building,
        Collector,
        Name::new("Collector"),
        IntersectorType::Emitter,
    );
    commands.spawn(collector).id()
}

fn animate_transform_system(
    mut ev_despawn: EventWriter<AnimationCompleteEvent>,
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut AnimateTransform, &mut Transform)>,
) {
    for (entity, mut animation, mut transform) in query.iter_mut() {
        animation.elapsed += time.delta_seconds();
        let t = animation.elapsed / animation.duration;
        if t >= 1.0 {
            commands.entity(entity).remove::<AnimateTransform>();
            ev_despawn.send(AnimationCompleteEvent(entity));
        } else {
            transform.translation = transform.translation.lerp(animation.target_position, t);
            transform.scale = transform.scale.lerp(animation.target_scale, t);
        }
    }
}

fn on_building_destroy(
    mut commands: Commands,
    mut grid_map: ResMut<GridMap>,
    mut ev_despawn: EventReader<AnimationCompleteEvent>,
    q_grid_pos: Query<(Entity, &GridPosition), (With<Building>, With<DeletionPending>)>,
) {
    for ev in ev_despawn.read() {
        if let Ok((entity, grid_position)) = q_grid_pos.get(ev.0) {
            grid_map.remove(GridLayer::Build, *grid_position).unwrap();
            commands.entity(entity).despawn();
        }
    }
}

fn destroy_block_system(
    mut commands: Commands,
    grid_map: Res<GridMap>,
    buttons: Res<ButtonInput<MouseButton>>,
    mouse_grid_pos: Res<MouseGridPosition>,
    mut ev_laser_update: EventWriter<LaserUpdateEvent>,
    intersector_query: Query<(Entity, &IntersectorType)>,
) {
    if buttons.just_pressed(MouseButton::Right) {
        let grid_pos = GridPosition::from(mouse_grid_pos.0);
        if let Some(entity) = grid_map.get(GridLayer::Build, grid_pos) {
            commands.entity(*entity).insert((
                AnimateTransform {
                    target_scale: Vec3::splat(0.0),
                    target_position: Vec3::new(
                        grid_pos.x as f32 * GRID_SCALE,
                        -0.5,
                        grid_pos.y as f32 * GRID_SCALE,
                    ),
                    duration: 0.5,
                    ..default()
                },
                DeletionPending,
            ));

            if let Ok((_, intersector_type)) = intersector_query.get(*entity) {
                ev_laser_update.send(LaserUpdateEvent {
                    entity: *entity,
                    update_type: UpdateType::Remove,
                    intersector: *intersector_type,
                    grid_position: grid_pos,
                });
            }
        }
    }
}
