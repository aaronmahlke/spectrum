use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    input::mouse::{MouseMotion, MouseWheel},
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
        app.add_systems(Startup, spawn_camera);
        app.add_systems(Update, (move_camera_system, zoom_camera));
    }
}

#[derive(Component)]
pub struct MainCamera;

fn move_camera_system(
    primary_windows: Query<&Window, With<PrimaryWindow>>,
    mut ev_motion: EventReader<MouseMotion>,
    input_mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut Transform, &Projection), With<Camera>>,
) {
    let primary_window = primary_windows.single();
    let window = Vec2::new(primary_window.width(), primary_window.height());

    let mut pan = Vec2::ZERO;

    for ev in ev_motion.read() {
        pan = ev.delta;
    }

    if input_mouse.pressed(MouseButton::Right) {
        for (mut transform, projection) in &mut query {
            if let Projection::Orthographic(projection) = projection {
                pan *= Vec2::new(projection.area.width(), projection.area.height()) / window;
            }

            if let Projection::Perspective(projection) = projection {
                pan *= Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / window;
            }

            let right = transform.rotation * Vec3::X * -pan.x;
            let up = transform.rotation * Vec3::Y * pan.y;
            let translation = right + up;

            transform.translation += translation;
        }
    }
}

fn zoom_camera(
    mut ev_scroll: EventReader<MouseWheel>,
    mut query: Query<&mut Projection, With<Camera>>,
) {
    let max_scale = 3.5;
    let min_scale = 10.0;
    let zoom_speed = 0.1;

    let mut zoom_delta = 0.0;
    for event in ev_scroll.read() {
        zoom_delta += event.y;
    }

    if zoom_delta == 0. {
        return;
    }

    for mut projection in &mut query {
        if let Projection::Orthographic(projection) = &mut *projection {
            projection.scale -= zoom_delta * projection.scale * 0.2 * zoom_speed;

            // clamp scale
            projection.scale = f32::max(projection.scale, max_scale);
            projection.scale = f32::min(projection.scale, min_scale);
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    let translation = Vec3::new(5.0, 5.0, 5.0);

    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },

            transform: Transform::from_translation(translation).looking_at(Vec3::ZERO, Vec3::Y),
            projection: Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical(1.0),
                scale: 4.0,
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
