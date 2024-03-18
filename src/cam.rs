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
