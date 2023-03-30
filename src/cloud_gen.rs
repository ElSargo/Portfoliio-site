use crate::noise::noise;
use crate::sdf::sdf as cloud_sdf;
use bevy::math::{vec2, vec3, vec4};
use bevy::prelude::{Mat3, Vec2, Vec3};

// A 16^3 chunk with 1-voxel boundary padding.

// This chunk will cover just a single octant of a sphere SDF (radius 15).
pub fn new(buffer_dimensions: [usize; 3]) -> Vec<Vec<Vec<Vec2>>> {
    let resolution = vec3(
        buffer_dimensions[0] as f32,
        buffer_dimensions[1] as f32,
        buffer_dimensions[2] as f32,
    );

    let mut data = vec![
        vec![vec![vec2(1., 1.); buffer_dimensions[2]]; buffer_dimensions[1]];
        buffer_dimensions[0]
    ];
    for x in 0..buffer_dimensions[0] {
        for y in 0..buffer_dimensions[1] {
            for z in 0..buffer_dimensions[2] {
                let p = coord_to_pos([x, y, z], resolution);
                let d = cloud_sdf(p);
                data[x][y][z] = vec2(if d > 0.0 { d } else { -noise(p).abs() }, 0.);
            }
        }
    }
    // Sun light info requires sdf and density info

    let sun_base = vec3(0., 1., 1.).normalize();

    let mut sun_directions = Vec::with_capacity(27);
    // sun_directions.push((sun_base, mie(1.)));
    let theta = 0.15;
    let angles = [-theta, 0.0, theta];
    for x in angles {
        for y in angles {
            for z in angles {
                let direction = rotate(sun_base, x, y, z);
                sun_directions.push((direction, mie(sun_base.dot(direction))));
            }
        }
    }
    for x in 0..buffer_dimensions[0] {
        for y in 0..buffer_dimensions[1] {
            for z in 0..buffer_dimensions[2] {
                let mut total = 0.;
                let dt = 2. / resolution.x.min(resolution.y).min(resolution.z);
                for (sun_direction, phase) in sun_directions.iter() {
                    let mut t = 1.;
                    let mut p = coord_to_pos([x, y, z], resolution);
                    let mut sample_point = [x, y, z];
                    while let Some(samp) = data
                        .get(sample_point[0])
                        .and_then(|slice| slice.get(sample_point[1]))
                        .and_then(|row| row.get(sample_point[2]))
                    {
                        if p.x.abs() > 1. || p.y.abs() > 1. || p.z.abs() > 1. {
                            break;
                        }

                        let distance = samp.x;
                        // let noise = mix(0_f32.max(samp.y + samp.w), 1., 0.5);
                        if distance < 0.0 {
                            t *= (-20. * dt * -distance).min(0.).exp();
                        }
                        p += *sun_direction * dt * *phase;
                        sample_point = pos_to_coord(p, resolution);
                    }
                    total += t;
                }
                data[x][y][z].y = total / sun_directions.len() as f32;
            }
        }
    }
    data
}

#[allow(dead_code)]
fn rotate(v: Vec3, x: f32, y: f32, z: f32) -> Vec3 {
    Mat3::from_euler(bevy::prelude::EulerRot::XYZ, x, y, z) * v
}

fn mie(costh: f32) -> f32 {
    // This function was optimized to minimize (delta*delta)/reference in order to capture
    // the low intensity behavior.
    let params = [
        9.805233e-06,
        -6.500000e+01,
        -5.500000e+01,
        8.194068e-01,
        1.388198e-01,
        -8.370334e+01,
        7.810083e+00,
        2.054747e-03,
        2.600563e-02,
        -4.552125e-12,
    ];

    let p1 = costh + params[3];
    let exp_values = vec4(
        params[1] * costh + params[2],
        params[5] * p1 * p1,
        params[6] * costh,
        params[9] * costh,
    )
    .exp();
    let exp_val_weight = vec4(params[0], params[4], params[7], params[8]);
    return exp_values.dot(exp_val_weight) * 0.25;
}

fn coord_to_pos<T: AsF32 + Copy>(coord: [T; 3], res: Vec3) -> Vec3 {
    (vec3(coord[0].as_f32(), coord[1].as_f32(), coord[2].as_f32()) / res - 0.5) * 2.
}

fn pos_to_coord(p: Vec3, res: Vec3) -> [usize; 3] {
    let c = ((p / 2.) + 0.5) * res;
    [
        c.x.round() as usize,
        c.y.round() as usize,
        c.z.round() as usize,
    ]
}

trait AsF32 {
    fn as_f32(self) -> f32;
}
impl AsF32 for f32 {
    fn as_f32(self) -> f32 {
        self
    }
}
impl AsF32 for u32 {
    fn as_f32(self) -> f32 {
        self as f32
    }
}
impl AsF32 for usize {
    fn as_f32(self) -> f32 {
        self as f32
    }
}
