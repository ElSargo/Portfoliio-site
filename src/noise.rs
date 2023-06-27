#![allow(dead_code)]

use bevy::{
    math::{mat2, mat3, vec2, vec3, vec4, Vec3Swizzles},
    prelude::{Mat2, Mat3, Vec3, Vec4},
};

fn hash(p: Vec3) -> f32 {
    // replace this by something better {
    let mut p = (p * 0.3183099 + 0.1).fract();
    p *= 17.0;
    return (p.x * p.y * p.z * (p.x + p.y + p.z)).fract();
}

pub fn value_noise(x: Vec3) -> f32 {
    let i = x.floor();
    let w = x.fract();
    // cubic interpolation
    let u = w * w * (3.0 - 2.0 * w);
    let a = hash(i + vec3(0.0, 0.0, 0.0));
    let b = hash(i + vec3(1.0, 0.0, 0.0));
    let c = hash(i + vec3(0.0, 1.0, 0.0));
    let d = hash(i + vec3(1.0, 1.0, 0.0));
    let e = hash(i + vec3(0.0, 0.0, 1.0));
    let f = hash(i + vec3(1.0, 0.0, 1.0));
    let g = hash(i + vec3(0.0, 1.0, 1.0));
    let h = hash(i + vec3(1.0, 1.0, 1.0));

    let k0 = a;
    let k1 = b - a;
    let k2 = c - a;
    let k3 = e - a;
    let k4 = a - b - c + d;
    let k5 = a - c - e + g;
    let k6 = a - b - e + f;
    let k7 = -a + b + c - d + e - f - g + h;

    return k0
        + k1 * u.x
        + k2 * u.y
        + k3 * u.z
        + k4 * u.x * u.y
        + k5 * u.y * u.z
        + k6 * u.z * u.x
        + k7 * u.x * u.y * u.z;
}

pub fn value_fbm(p: Vec3) -> f32 {
    let mut p = p;
    let mut t = 0.;
    let mut s = 1.;
    let mut c = 1.;

    for _ in 0..4 {
        // p += vec3(13.123, -72., 234.23);
        t += noised(p * s).x * c;
        s *= 2.;
        c *= 0.5;
    }
    return t / 2.7;
}

const ROTATE: Mat3 = mat3(
    vec3(0.00, 1.60, 1.20),
    vec3(-1.60, 0.72, -0.96),
    vec3(-1.20, -0.96, 1.28),
);

fn hash33(p3: Vec3) -> Vec3 {
    let mut p = p3;
    p = (p * vec3(0.1031, 0.1030, 0.0973)).fract();
    p += p.dot(p.yxz() + 33.33);
    return ((p.xxy() + p.yxx()) * p.zyx()).fract();
}

fn remap(x: f32, a: f32, b: f32, c: f32, d: f32) -> f32 {
    return (((x - a) / (b - a)) * (d - c)) + c;
}

pub fn noised(x: Vec3) -> Vec4 {
    let i = x.floor();
    let w = x.fract();
    // cubic interpolation
    let u = w * w * (3.0 - 2.0 * w);
    let du = 6.0 * w * (1.0 - w);
    let a = hash(i + vec3(0.0, 0.0, 0.0));
    let b = hash(i + vec3(1.0, 0.0, 0.0));
    let c = hash(i + vec3(0.0, 1.0, 0.0));
    let d = hash(i + vec3(1.0, 1.0, 0.0));
    let e = hash(i + vec3(0.0, 0.0, 1.0));
    let f = hash(i + vec3(1.0, 0.0, 1.0));
    let g = hash(i + vec3(0.0, 1.0, 1.0));
    let h = hash(i + vec3(1.0, 1.0, 1.0));

    let k0 = a;
    let k1 = b - a;
    let k2 = c - a;
    let k3 = e - a;
    let k4 = a - b - c + d;
    let k5 = a - c - e + g;
    let k6 = a - b - e + f;
    let k7 = -a + b + c - d + e - f - g + h;

    let deriv = du
        * vec3(
            k1 + k4 * u.y + k6 * u.z + k7 * u.y * u.z,
            k2 + k5 * u.z + k4 * u.x + k7 * u.z * u.x,
            k3 + k6 * u.x + k5 * u.y + k7 * u.x * u.y,
        );
    return vec4(
        k0 + k1 * u.x
            + k2 * u.y
            + k3 * u.z
            + k4 * u.x * u.y
            + k5 * u.y * u.z
            + k6 * u.z * u.x
            + k7 * u.x * u.y * u.z,
        deriv.x,
        deriv.y,
        deriv.z,
    );
}

pub fn fbmd(mut p: Vec3) -> Vec4 {
    let mut t = Vec4::ZERO;
    let mut s = 1.;
    let mut c = 1.;

    for i in 0..4 {
        p += vec3(13.123, -72., 234.23);
        let n = noised(p * s) * c;
        t.x += n.x;
        if i < 1 {
            t.y += n.y;
            t.z += n.z;
            t.w += n.w;
        }
        s *= 2.;
        c *= 0.5;

        let rot = rotate(2.135532) * p.xz();
        p = vec3(rot.x, p.y, rot.y);
        let rot = rotate(1.5532) * p.yz();
        p = vec3(p.x, rot.x, rot.y);
    }
    return t;
}

pub fn worley_noise(p: Vec3, f: Vec3) -> f32 {
    let id = p.floor();

    let p = p.fract();

    let mut min_dist = 10000_f32;
    for x in [-1., 0., 1.] {
        for y in [-1., 0., 1.] {
            for z in [-1., 0., 1.] {
                let offset = vec3(x, y, z);
                let mut h = hash33((id + offset) % f) * 0.5 + 0.5;
                h += offset;
                let d = p - h;
                min_dist = min_dist.min(d.dot(d));
            }
        }
    }

    return min_dist.sqrt();
}

pub fn wfbm(p: Vec3, f: Vec3) -> f32 {
    let mut p = p + vec3(100.123, -12.24245, 13.414);
    let mut t = 0.0;
    let mut s = 1.;
    let mut c = 1.;

    for _ in 0..12 {
        p += vec3(13.123, -72., 234.23);
        let n = worley_noise(p * s, f);
        t += n * c;
        s *= 2.;
        c *= 0.5;
        let rot = rotate(2.135532) * p.xz();
        p = vec3(rot.x, p.y, rot.y);
        let rot = rotate(1.135532) * p.yz();
        p = vec3(p.x, rot.x, rot.y);
    }
    return 4.0 - t;
}

fn rotate(a: f32) -> Mat2 {
    let (s, c) = a.sin_cos();
    mat2(vec2(c, -s), vec2(s, c))
}
