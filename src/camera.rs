use std::f32::consts::{E, PI};

use bevy::{math::vec3, prelude::*};

use crate::noise::value_noise;

#[derive(Component)]
pub struct CameraController {}

impl Default for CameraController {
    fn default() -> Self {
        Self {}
    }
}

#[allow(dead_code)]
pub fn camera_controller(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<CameraController>>,
) {
    let dt = time.elapsed_seconds();

    if let Ok(mut transform) = query.get_single_mut() {
        transform.rotation = Quat::from_euler(
            EulerRot::XYZ,
            (value_noise(vec3(dt, dt * PI, dt * E) * 0.1) - 0.5) * 0.2 - 1.5,
            (value_noise(vec3(dt * E, dt, dt * PI) * 0.1) - 0.5) * 0.2,
            (value_noise(vec3(dt * PI, dt * E, dt) * 0.1) - 0.5) * 0.2 + PI,
        );
    }
}
