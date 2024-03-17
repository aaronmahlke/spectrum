use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    diagnostic::FrameTimeDiagnosticsPlugin,
    pbr::PointLightShadowMap,
    prelude::*,
    render::{
        camera::{Exposure, ScalingMode},
        view::ColorGrading,
    },
    window::{PresentMode, PrimaryWindow},
};

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use fps::FPSPlugin;
mod fps;

fn main() {
    let mut app = App::new();

    // plugins
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "bevyaac".into(),
            present_mode: PresentMode::Immediate,
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
    }))
    .add_plugins(WorldInspectorPlugin::new())
    .add_plugins((FPSPlugin, FrameTimeDiagnosticsPlugin));

    // resources
    app.insert_resource(ClearColor(Color::BLACK))
        .insert_resource(PointLightShadowMap { size: 2048 })
        .insert_resource(AmbientLight {
            brightness: 0.0,
            ..default()
        })
        .insert_resource(Msaa::default())
        .insert_resource(MouseWorldPosition(Vec2::ZERO))
        .insert_resource(MouseGridPosition(Vec2::ZERO));

    // systems
    app.add_systems(Startup, (setup, spawn_color_wells));
    app.add_systems(
        Update,
        (
            cursor_system,
            update_cursor_attachment,
            place_collector,
            animate_transform_system,
            destoy_block_system,
        ),
    );
    app.run();
}

const GRID_SIZE: i32 = 10;
const GRID_SCALE: f32 = 1.0;

#[derive(Component)]
struct GridPosition {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Laser;

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

#[derive(Component)]
struct AnimateMaterial {
    target_color: Color,
    target_emissive: Color,
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
struct CursorAttachement;

#[derive(Component)]
struct MainCamera;

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

fn spawn_color_wells(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for x in -GRID_SIZE..GRID_SIZE {
        for y in -GRID_SIZE..GRID_SIZE {
            let should_spawn = rand::random::<f32>();
            let position = Vec3::new(x as f32 * GRID_SCALE, -0.49, y as f32 * GRID_SCALE);
            let color = Color::ORANGE_RED;
            let chance = 0.03;
            if should_spawn > chance {
                continue;
            }
            commands.spawn((
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
                GridPosition { x, y },
                ColorWell { color },
                Name::new("Color Well"),
            ));
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cylinder_mesh = meshes.add(Cylinder::new(0.5, 2.0).mesh().resolution(50));
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

    // camera
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            transform: Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            projection: Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical(6.0),
                ..default()
            }),
            color_grading: ColorGrading {
                post_saturation: 1.2,
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            exposure: Exposure { ev100: 6.0 },
            ..default()
        },
        BloomSettings {
            ..Default::default()
        },
        MainCamera,
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
        CursorAttachement,
    ));
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

fn update_cursor_attachment(
    mut cursor_attachement: Query<&mut Transform, With<CursorAttachement>>,
    mouse_grid_pos: Res<MouseGridPosition>,
) {
    for mut transform in cursor_attachement.iter_mut() {
        transform.translation = transform.translation.lerp(
            Vec3::new(
                mouse_grid_pos.0.x,
                transform.translation.y,
                mouse_grid_pos.0.y,
            ),
            0.1,
        );
    }
}

fn world_to_grid(world_position: Vec2) -> Vec2 {
    Vec2::new(
        (world_position.x / GRID_SCALE).round(),
        (world_position.y / GRID_SCALE).round(),
    )
}

fn grid_to_world(grid_position: Vec2) -> Vec2 {
    Vec2::new(grid_position.x * GRID_SCALE, grid_position.y * GRID_SCALE)
}

fn place_collector(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    mouse_grid_pos: Res<MouseGridPosition>,
    mut q_grid_pos: Query<(Entity, &GridPosition, &mut Handle<StandardMaterial>), With<ColorWell>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        // Left button was pressed
        for (well_entity, grid_pos, mut mat_handle) in &mut q_grid_pos {
            if grid_pos.x as f32 == mouse_grid_pos.0.x && grid_pos.y as f32 == mouse_grid_pos.0.y {
                commands.entity(well_entity).insert(Active);

                let collector = commands
                    .spawn((
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
                            transform: Transform::from_translation(Vec3::new(
                                grid_pos.x as f32 * GRID_SCALE,
                                -0.4,
                                grid_pos.y as f32 * GRID_SCALE,
                            )),
                            ..Default::default()
                        },
                        AnimateTransform {
                            target_position: Vec3::new(
                                grid_pos.x as f32 * GRID_SCALE,
                                0.5,
                                grid_pos.y as f32 * GRID_SCALE,
                            ),
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
                    ))
                    .with_children(|parent| {
                        let laser_length = 10.0;
                        parent.spawn((
                            PbrBundle {
                                mesh: meshes.add(Cylinder::new(0.5, 2.0).mesh().resolution(50)),
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
                                    .with_scale(Vec3::new(0.04, 0.0, 0.04))
                                    .with_rotation(Quat::from_rotation_x(
                                        -std::f32::consts::FRAC_PI_2,
                                    )),
                                ..Default::default()
                            },
                            AnimateTransform {
                                target_scale: Vec3::new(0.03, laser_length, 0.04),
                                target_position: Vec3::new(0.0, 0.2, laser_length + 0.5),
                                duration: 2.5,
                                ..Default::default()
                            },
                            Laser,
                            Name::new("Collector Laser"),
                        ));
                    });
            }
        }
    }
}

fn animate_transform_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut AnimateTransform, &mut Transform)>,
) {
    for (entity, mut animation, mut transform) in query.iter_mut() {
        animation.elapsed += time.delta_seconds();
        let t = animation.elapsed / animation.duration;
        if t >= 1.0 {
            commands.entity(entity).remove::<AnimateTransform>();
        } else {
            transform.translation = transform.translation.lerp(animation.target_position, t);
            transform.scale = transform.scale.lerp(animation.target_scale, t);
        }
    }
}

fn destoy_block_system(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    mouse_grid_pos: Res<MouseGridPosition>,
    q_grid_pos: Query<(Entity, &GridPosition), (With<Building>, Without<ColorWell>)>,
) {
    if buttons.just_pressed(MouseButton::Right) {
        println!("Right button was pressed");
        for (entity, grid_pos) in &q_grid_pos {
            if grid_pos.x as f32 == mouse_grid_pos.0.x && grid_pos.y as f32 == mouse_grid_pos.0.y {
                commands.entity(entity).insert(AnimateTransform {
                    target_scale: Vec3::splat(0.0),
                    target_position: Vec3::new(
                        grid_pos.x as f32 * GRID_SCALE,
                        -0.5,
                        grid_pos.y as f32 * GRID_SCALE,
                    ),
                    duration: 0.5,
                    elapsed: 0.0,
                    ..default()
                });
            }
        }
    }
}
