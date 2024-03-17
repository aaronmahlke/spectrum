use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    prelude::*,
    render::{
        camera::{Exposure, ScalingMode},
        view::ColorGrading,
    },
    window::PrimaryWindow,
};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera_system);
        app.add_systems(Update, camera_pan);
    }
}

#[derive(Component)]
pub struct MainCamera;

fn setup_camera_system(mut commands: Commands) {
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
}

fn camera_pan(
    buttons: Res<ButtonInput<MouseButton>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    mut cursor: EventReader<CursorMoved>,
) {
    if buttons.pressed(MouseButton::Right) {
        for cursor_event in cursor.read() {
            if let Some(delta) = cursor_event.delta {
                for mut transform in &mut camera_query {
                    let forward = transform.local_y();
                    let right = transform.local_x();
                    println!("Local: {:?}, {:?}", forward, right);
                    println!("Delta: {:?}", delta);
                    println!("Translation: {:?}", transform.translation);
                    transform.translation +=
                        right * delta.x / 100.0 * -1.0 + forward * delta.y / 100.0;
                }
            }
        }
    }
}

fn camera_movement(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut camera_query: Query<(&mut Transform, &OrthographicProjection)>,
    mut last_pos: Local<Option<Vec2>>,
) {
    let window = primary_window.single();
    let window_size = Vec2::new(window.width(), window.height());

    // Use position instead of MouseMotion, otherwise we don't get acceleration movement
    let current_pos = match window.cursor_position() {
        Some(c) => Vec2::new(c.x, -c.y),
        None => return,
    };
    let delta_device_pixels = current_pos - last_pos.unwrap_or(current_pos);

    for (mut transform, projection) in &mut camera_query {
        println!("Camera");
        if mouse_buttons.pressed(MouseButton::Right) {
            // let proj_size = projection.area.size();
            let proj_size = projection.area.size();
            // let proj_size = 6.0;

            let world_units_per_device_pixel = proj_size / window_size;

            // The proposed new camera position
            let delta_world = delta_device_pixels * world_units_per_device_pixel;
            let proposed_cam_transform = transform.translation - delta_world.extend(0.);

            transform.translation = proposed_cam_transform;
        }
    }
    *last_pos = Some(current_pos);
}
