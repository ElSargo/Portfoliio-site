use crate::sdf::sdf as cloud_sdf;
use bevy::math::vec3;
use bevy::prelude::Vec3;

// A 16^3 chunk with 1-voxel boundary padding.
const SUN: Vec3 = vec3(1., 1., 1.);

// This chunk will cover just a single octant of a sphere SDF (radius 15).
pub fn new(res: f32) -> Vec<Vec3> {
    let buffer_side_length = res as usize;
    let buffer_size = buffer_side_length.pow(3);
    let mut sdf = vec![1.0; buffer_size];
    for [x, y, z] in (0..buffer_side_length.pow(3)).map(|i| delinearize(buffer_side_length, i)) {
        let p = coord_to_pos([x, y, z], res);
        let d = cloud_sdf(p);
        // println!("{p}, {d}");
        sdf[linearize(buffer_side_length, [x, y, z])] = d;
    }

    let mut sun_light_distacnces = Vec::with_capacity(sdf.len());
    for [x, y, z] in (0..buffer_side_length.pow(3)).map(|i| delinearize(buffer_side_length, i)) {
        // March to the sun
        let mut p = coord_to_pos([x as u32, y as u32, z as u32], res);
        let mut t = 0.;
        while let Some(dist) = get_dist(p, &sdf, res, buffer_side_length) {
            let d = dist.abs();
            if dist < 0.0 {
                t += d;
            }
            p += SUN * (d + 0.1);
        }
        sun_light_distacnces.push([t, 0.0]);
    }

    todo!()
}

fn linearize(buffer_side_length: usize, [x, y, z]: [usize; 3]) -> usize {
    x * buffer_side_length * buffer_side_length + y * buffer_side_length + z
}

fn delinearize(buffer_side_length: usize, mut i: usize) -> [usize; 3] {
    let area = buffer_side_length.pow(2);
    let x = i / area;
    i -= x * area;
    let y = i / buffer_side_length;
    let z = i - y;
    [x, y, z]
}

fn get_dist(p: Vec3, sdf: &[f32], res: f32, buffer_side_length: usize) -> Option<f32> {
    let [x, y, z] = pos_to_coord(p, res);
    sdf.get(linearize(buffer_side_length, [x, y, z]) as usize)
        .copied()
}

fn coord_to_pos<T: AsF32 + Copy>(coord: [T; 3], res: f32) -> bevy::prelude::Vec3 {
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
