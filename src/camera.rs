use std::process::CommandEnvs;

use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    prelude::*,
    render::{
        camera::{Exposure, ScalingMode},
        view::ColorGrading,
    },
};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera_system);
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

fn camera_pan(buttons: Res<ButtonInput<MouseButton>>) {
    if buttons.pressed(MouseButton::Right) {
        println!("Right mouse button pressed");
    }
}
