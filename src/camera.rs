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
        app.add_systems(Update, move_camera_system);
    }
}

#[derive(Component)]
pub struct MainCamera;

/// Tags an entity as capable of panning and orbiting.
#[derive(Component)]
struct PanOrbitCamera {
    /// The "focus point" to orbit around. It is automatically updated when panning the camera
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
    pub zoom_upper_limit: Option<f32>,
    pub zoom_lower_limit: Option<f32>,
}

impl Default for PanOrbitCamera {
    fn default() -> Self {
        PanOrbitCamera {
            focus: Vec3::ZERO,
            radius: 5.0,
            upside_down: false,
            zoom_upper_limit: None,
            zoom_lower_limit: None,
        }
    }
}

fn move_camera_system(
    primary_windows: Query<&Window, With<PrimaryWindow>>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform, &Projection)>,
) {
    let primary_window = primary_windows.single();
    let window = Vec2::new(primary_window.width(), primary_window.height());
}

/// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with right mouse click.
fn pan_orbit_camera(
    primary_windows: Query<&Window, With<PrimaryWindow>>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform, &Projection)>,
) {
    let primary_window = primary_windows.single();
    let window = Vec2::new(primary_window.width(), primary_window.height());

    // change input mapping for orbit and panning here
    let pan_button = MouseButton::Right;

    let mut pan = Vec2::ZERO;
    let mut scroll = 0.0;

    if input_mouse.pressed(pan_button) {
        for ev in ev_motion.read() {
            pan += ev.delta;
        }
    }
    for ev in ev_scroll.read() {
        scroll += ev.y;
    }

    for (mut pan_orbit, mut transform, projection) in query.iter_mut() {
        let mut any = false;
        if pan.length_squared() > 0.0 {
            any = true;
            // make panning distance independent of resolution and FOV,
            //

            match *projection {
                Projection::Perspective(ref p) => {
                    pan *= Vec2::new(p.fov * p.aspect_ratio, p.fov) / window;
                }
                Projection::Orthographic(ref p) => {
                    pan *= Vec2::new(p.area.width(), p.area.height()) / window;
                }
            }

            // if let Projection::Perspective(projection) = projection {
            //     pan *= Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / window;
            // }
            //
            // if let Projection::Orthographic(projection) = projection {
            //     pan *= Vec2::new(projection.area.width(), projection.area.height()) / window;
            // }

            // translate by local axes
            let right = transform.rotation * Vec3::X * -pan.x;
            let up = transform.rotation * Vec3::Y * pan.y;
            // make panning proportional to distance away from focus point
            let translation = (right + up) * pan_orbit.radius;
            pan_orbit.focus += translation;
        }

        // if scroll.abs() > 0.0 {
        //     any = true;
        //     pan_orbit.radius -= scroll * pan_orbit.radius * 0.2;
        //     // dont allow zoom to reach zero or you get stuck
        //     pan_orbit.radius = f32::max(pan_orbit.radius, 0.05);
        // }

        if any {
            // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
            // parent = x and y rotation
            // child = z-offset
            let rot_matrix = Mat3::from_quat(transform.rotation);
            transform.translation =
                pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
        }
    }

    // consume any remaining events, so they don't pile up if we don't need them
    // (and also to avoid Bevy warning us about not checking events every frame update)
    ev_motion.clear();
}

/// Spawn a camera like this
fn spawn_camera(mut commands: Commands) {
    let translation = Vec3::new(5.0, 5.0, 5.0);
    let radius = translation.length();

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
        PanOrbitCamera {
            radius,
            ..Default::default()
        },
        MainCamera,
    ));
}
