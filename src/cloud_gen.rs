use std::ops::{Add, Mul};

use crate::noise::noise;
use crate::sdf::sdf as cloud_sdf;
use bevy::math::{vec3, vec4};
use bevy::prelude::{Vec3, Vec4};

// A 16^3 chunk with 1-voxel boundary padding.

// This chunk will cover just a single octant of a sphere SDF (radius 15).
pub fn new(res: f32) -> Vec<Vec4> {
    assert_eq!(linearize(100, [4, 0, 0]), 4);
    assert_eq!(linearize(100, [4, 1, 0]), 104);
    assert_eq!(linearize(100, [4, 1, 1]), 10104);

    assert_eq!(delinearize(100, 4), [4, 0, 0]);
    assert_eq!(delinearize(100, 104), [4, 1, 0]);
    assert_eq!(delinearize(100, 10104,), [4, 1, 1]);

    assert_eq!(linearize(100, [0, 50, 0]), 50 * 100);
    assert_eq!(delinearize(100, 50 * 100), [0, 50, 0]);

    let buffer_side_length = res as usize;
    let buffer_size = buffer_side_length.pow(3);
    let mut sdf = vec![vec4(1., 1., 1., 1.); buffer_size];
    for (i, [x, y, z]) in (0..buffer_size).map(|i| (i, delinearize(buffer_side_length, i))) {
        let p = coord_to_pos([x, y, z], res);
        let d = cloud_sdf(p);
        let n = noise(p * 1.);
        let n2 = noise(p * 2.);
        // println!("{p}, {d}");
        sdf[i] = vec4(d, n, 0., n2);
    }
    // Sun light info requires sdf and density info

    println!("Marching");
    for (i, [x, y, z]) in (0..buffer_size).map(|i| (i, delinearize(buffer_side_length, i))) {
        //                     y
        let sun_base = vec3(1., 0., 1.).normalize();
        // March to the sun
        let mut total = 0.;
        let dt = 2. / res;
        for sun in [
            sun_base.add(vec3(0.1, 0., 0.1)).normalize(),
            sun_base.add(vec3(-0.1, 0., 0.1)).normalize(),
            sun_base.add(vec3(0.1, 0., -0.1)).normalize(),
            sun_base.add(vec3(-0.1, 0., -0.1)).normalize(),
            sun_base,
        ] {
            let mut t = 1.;
            let mut p = coord_to_pos([x, y, z], res);
            while let Some(samp) = sdf.get(linearize(buffer_side_length, pos_to_coord(p, res))) {
                if p.x.abs() > 1. || p.y.abs() > 1. || p.z.abs() > 1. {
                    break;
                }
                if i == 1 {
                    println! {"{:?} [:] {t} [:] {:?}",p,pos_to_coord(p, res)};
                }
                let distance = samp.x;
                let noise = mix(0_f32.max(samp.y + samp.w), 1., 0.5);
                if distance < 0.05 + noise * 0.1 {
                    t *= (noise * -7. * dt).min(0.).exp();
                }
                p += sun * dt;
            }
            total += t;
        }
        sdf[i].z = total * 0.2;
    }
    sdf
}

fn mix(a: f32, b: f32, t: f32) -> f32 {
    a * (1. - t) + b * t
}

fn linearize(buffer_side_length: usize, [x, y, z]: [usize; 3]) -> usize {
    x + buffer_side_length.mul(y) + (buffer_side_length * buffer_side_length).mul(z)
}

fn delinearize(buffer_side_length: usize, mut i: usize) -> [usize; 3] {
    let z = i / (buffer_side_length * buffer_side_length);
    i -= z * (buffer_side_length * buffer_side_length);
    let y = i / buffer_side_length;
    let x = i % buffer_side_length;
    [x, y, z]
}

fn coord_to_pos<T: AsF32 + Copy>(coord: [T; 3], res: f32) -> Vec3 {
    (vec3(coord[0].as_f32(), coord[1].as_f32(), coord[2].as_f32()) / res - 0.5) * 2.
}

fn pos_to_coord(p: Vec3, res: f32) -> [usize; 3] {
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
