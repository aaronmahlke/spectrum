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
    app.add_systems(Update, (cursor_system));
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
struct Cursor;

#[derive(Component)]
struct MainCamera;

#[derive(Resource, Default)]
struct MouseWorldPosition(Vec2);

#[derive(Resource, Default)]
struct MouseGridPosition(Vec2);

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
            let chance = 0.05;
            if should_spawn > chance {
                continue;
            }
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                    material: materials.add(StandardMaterial {
                        base_color: color,
                        reflectance: 0.5,
                        emissive: color * 20.0,
                        ..default()
                    }),
                    transform: Transform::from_translation(position),
                    ..Default::default()
                },
                GridPosition { x, y },
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
                illuminance: 500.0,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 1.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        Name::new("Directional Light"),
    ));

    // Floor
    let floor_mat = materials.add(StandardMaterial {
        base_color: Color::rgb(0.0, 0.0, 0.0),
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

    // Laser
    commands.spawn((
        PbrBundle {
            mesh: cylinder_mesh.clone(),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                specular_transmission: 0.9,
                diffuse_transmission: 1.0,
                thickness: 1.8,
                ior: 1.5,
                perceptual_roughness: 0.12,
                emissive: Color::ORANGE_RED * 10.0,
                ..default()
            }),
            transform: Transform::from_xyz(1.0, 0.0, 0.0)
                .with_scale(Vec3::new(0.03, 4.0, 0.03))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            ..default()
        },
        Name::new("Laser"),
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
        Cursor,
    ));
}

fn cursor_system(
    mut commands: Commands,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut q_cursor_block: Query<&mut Transform, With<Cursor>>,
    mut mouse_world_position: ResMut<MouseWorldPosition>,
    mut mouse_grid_position: ResMut<MouseGridPosition>,
) {
    let (camera, camera_transform) = q_camera.single();
    let window = q_window.single();

    let mut cursor_transform = q_cursor_block.single_mut();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| {
            println!("ray: {:?}", ray);
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
        println!("world_position: {:?}", world_position);
        println!(
            "grid_position: {:?}",
            world_to_grid(Vec2::new(world_position.x, world_position.z))
        );

        mouse_world_position.0 = Vec2::new(world_position.x, world_position.z);
        mouse_grid_position.0 = world_to_grid(Vec2::new(world_position.x, world_position.z));

        // slowly move the cursor block to the target position
        let grid_position = world_to_grid(Vec2::new(world_position.x, world_position.z));
        cursor_transform.translation = cursor_transform
            .translation
            .lerp(Vec3::new(grid_position.x, 0.0, grid_position.y), 0.1);
    }
}

fn world_to_grid(world_position: Vec2) -> Vec2 {
    Vec2::new(
        (world_position.x / GRID_SCALE).round(),
        (world_position.y / GRID_SCALE).round(),
    )
}
