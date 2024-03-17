use bevy::math::{Vec2, Vec3};
use flo_curves::bezier::Curve;
use flo_curves::{BezierCurve, Coord2, Coordinate2D};

fn interpolate(from: f32, to: f32, t: f32) -> f32 {
    // todo: curve from resource
    let c = Curve {
        start_point: Coord2::from((0., 0.)),
        end_point: Coord2::from((1., 1.)),
        control_points: (Coord2::from((0.2, 0.75)), Coord2::from((0.68, 1.))),
    };

    let t = t.clamp(0.0, 1.0);
    let step = c.point_at_pos(t as f64).y() as f32;
    from + (to - from) * step
}

pub(crate) trait Interpolate {
    fn interpolate(&self, to: Self, t: f32) -> Self;
}

impl Interpolate for f32 {
    fn interpolate(&self, to: f32, t: f32) -> f32 {
        interpolate(*self, to, t) as f32
    }
}

impl Interpolate for Vec3 {
    fn interpolate(&self, to: Vec3, t: f32) -> Vec3 {
        Vec3::new(
            self.x.interpolate(to.x, t),
            self.y.interpolate(to.y, t),
            self.z.interpolate(to.z, t),
        )
    }
}

impl Interpolate for Vec2 {
    fn interpolate(&self, to: Vec2, t: f32) -> Vec2 {
        Vec2::new(self.x.interpolate(to.x, t), self.y.interpolate(to.y, t))
    }
}
