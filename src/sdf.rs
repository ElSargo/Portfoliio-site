pub fn sdf(position: Vec3) -> f32 {
    let d = sd_ellipsoid(position - vec3(0., -0.15, 0.), vec3(0.5, 0.2, 0.5));
    let fbm = sd_fbm(
        position * 2. + vec3(100.123, -23.13, 124.23),
        d, /*0.075*/
    ) / 2.;
    // let ma = mat3(
    //     vec3(0.00, 1.60, 1.20),
    //     vec3(-1.60, 0.72, -0.96),
    //     vec3(-1.20, -0.96, 1.28),
    // );
    // let fbm2 = sd_fbm(ma * position * 2. + vec3(-223.123, -1.13, -314.23), 0.075) / 2.;

    // smooth_min(
    //     smooth_max(smooth_min(fbm, fbm2, 0.01), d - 0.2, 0.1),
    //     d,
    //     0.2,
    // )
    fbm
}

fn scale<C: Fn(Vec3) -> f32>(p: Vec3, f: f32, sdf: C) -> f32 {
    sdf(p * f) / f
}

fn sd_grid_sphere(i: Vec3, f: Vec3, c: Vec3) -> f32 {
    // random radius at grid vertex i+c
    let rad = 0.5 * hash13(i + c);
    // distance to sphere at grid vertex i+c
    return (f - c).length() - rad;
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

fn sd_fbm(p: Vec3, d: f32) -> f32 {
    let mut p = p;
    let mut d = d;
    let mut s = 1.;
    for _ in 0..6 {
        // evaluate new octave
        {
            let mut n = s * sd_base(p);

            // add
            n = smooth_max(n, d - 0.1 * s, 0.3 * s);
            d = smooth_min(n, d, 0.3 * s);
        }
        {
            let mut n = s * sd_base(p + 100.);

            // add
            n = smooth_max(n, d - 0.1 * s, 0.3 * s);
            d = smooth_min(n, d, 0.3 * s);
        }

        // prepare next octave
        p = mat3(
            vec3(0.00, 1.60, 1.20),
            vec3(-1.60, 0.72, -0.96),
            vec3(-1.20, -0.96, 1.28),
        ) * p;

        s = 0.4 * s;
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

fn sd_ellipsoid(p: Vec3, r: Vec3) -> f32 {
    let k0 = (p / r).length();
    let k1 = (p / (r * r)).length();
    k0 * (k0 - 1.0) / k1
}

fn sd_sphere(p: Vec3, r: f32) -> f32 {
    p.length() - r
}

fn hash13(p: Vec3) -> f32 {
    let mut p = p.mul(0.1031).fract();
    p += p.dot(p.zyx() + 31.32);
    return ((p.x + p.y) * p.z).fract();
}

use bevy::{
    math::{mat3, vec3, Vec3Swizzles},
    prelude::Vec3,
};
use std::ops::Mul;
