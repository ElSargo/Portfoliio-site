pub fn sdf(position: Vec3) -> f32 {
    let d = sd_ellipsoid(position, vec3(0.5, 0.0, 0.5));
    let d = ((position - vec3(0., 0.2, 0.)).length() - 0.3).min(d);
    let fbm = sd_fbm(position + vec3(-100.123, 303.13, 634.23), d, 20);
    fbm // d
}

const ROTATE: Mat3 = mat3(
    vec3(0.00, 1.60, 1.20),
    vec3(-1.60, 0.72, -0.96),
    vec3(-1.20, -0.96, 1.28),
);

fn sd_box(p: Vec3, b: Vec3) -> f32 {
    let q = p.abs() - b;
    return q.max(vec3(0.0, 0.0, 0.0)).length() + q.x.max(q.y.max(q.z)).min(0.0);
}

fn sd_grid_sphere(i: Vec3, f: Vec3, c: Vec3) -> f32 {
    // random radius at grid vertex i+c
    let has = 0.5 * hash43(i + c);
    // distance to sphere at grid vertex i+c
    return (f - c + has.xyz()).length() - has.w - 0.2;
}

fn sd_base(p: Vec3) -> f32 {
    let i = p.floor();
    let f = p.fract();
    // distance to the 8 corners spheres

    sd_grid_sphere(i, f, vec3(0.0, 0.0, 0.0))
        .min(sd_grid_sphere(i, f, vec3(0.0, 0.0, 1.0)))
        .min(sd_grid_sphere(i, f, vec3(0.0, 1.0, 0.0)))
        .min(sd_grid_sphere(i, f, vec3(0.0, 1.0, 1.0)))
        .min(sd_grid_sphere(i, f, vec3(1.0, 0.0, 0.0)))
        .min(sd_grid_sphere(i, f, vec3(1.0, 0.0, 1.0)))
        .min(sd_grid_sphere(i, f, vec3(1.0, 1.0, 0.0)))
        .min(sd_grid_sphere(i, f, vec3(1.0, 1.0, 1.0)))
}

pub fn sd_fbm(p: Vec3, d: f32, octaves: i32) -> f32 {
    let mut p = p + vec3(122.133, -3.123, 9023.1);
    let mut d = d;
    let mut s = 1.;
    for _ in 0..octaves {
        // evaluate new octave
        {
            let mut n = s * sd_base(p);

            // // add
            n = smooth_max(n, d - 0.4 * s, s);
            d = smooth_min(n, d, 0.02 * s);
        }

        // prepare next octave
        p = ROTATE * p;

        s = 0.5 * s;
    }
    return d;
}

fn mix(a: f32, b: f32, t: f32) -> f32 {
    a * (1. - t) + b * t
}

fn smooth_max(a: f32, b: f32, k: f32) -> f32 {
    let h = (0.0_f32).max(k - (a - b).abs());
    a.max(b) + h * h * 0.25 / k
}

fn smooth_min(a: f32, b: f32, k: f32) -> f32 {
    let h = (0.5 + 0.5 * (b - a) / k).clamp(0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}

#[allow(dead_code)]
fn sd_ellipsoid(p: Vec3, r: Vec3) -> f32 {
    let k0 = (p / r).length();
    let k1 = (p / (r * r)).length();
    k0 * (k0 - 1.0) / k1
}

fn hash43(p: Vec3) -> Vec4 {
    let mut p4 = (p.xyzx() * vec4(0.1031, 0.1030, 0.0973, 0.1099)).fract();
    p4 += p4.dot(p4.wzxy() + 33.33);
    return ((p4.xxyz() + p4.yzzw()) * p4.zywx()).fract();
}

use bevy::{
    math::{mat3, vec3, vec4, Vec3Swizzles, Vec4Swizzles},
    prelude::{Mat3, Vec3, Vec4},
};
