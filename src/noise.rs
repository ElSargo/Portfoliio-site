use bevy::{
    math::{mat3, vec3, vec4, Vec3Swizzles, Vec4Swizzles},
    prelude::{Mat3, Vec3, Vec4},
};

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

// Gradient noise by iq (modified to be tileable)
fn gradient_noise(x: Vec3, freq: f32) -> f32 {
    // grid
    let p = x.floor();
    let w = x.fract();

    // quintic interpolant
    let u = w * w * w * (w * (w * 6. - 15.) + 10.);

    // gradients
    let ga = hash33(p + vec3(0., 0., 0.));
    let gb = hash33(p + vec3(1., 0., 0.));
    let gc = hash33(p + vec3(0., 1., 0.));
    let gd = hash33(p + vec3(1., 1., 0.));
    let ge = hash33(p + vec3(0., 0., 1.));
    let gf = hash33(p + vec3(1., 0., 1.));
    let gg = hash33(p + vec3(0., 1., 1.));
    let gh = hash33(p + vec3(1., 1., 1.));

    // projections
    let va = ga.dot(w - vec3(0., 0., 0.));
    let vb = gb.dot(w - vec3(1., 0., 0.));
    let vc = gc.dot(w - vec3(0., 1., 0.));
    let vd = gd.dot(w - vec3(1., 1., 0.));
    let ve = ge.dot(w - vec3(0., 0., 1.));
    let vf = gf.dot(w - vec3(1., 0., 1.));
    let vg = gg.dot(w - vec3(0., 1., 1.));
    let vh = gh.dot(w - vec3(1., 1., 1.));

    // interpolation
    return va
        + u.x * (vb - va)
        + u.y * (vc - va)
        + u.z * (ve - va)
        + u.x * u.y * (va - vb - vc + vd)
        + u.y * u.z * (va - vc - ve + vg)
        + u.z * u.x * (va - vb - ve + vf)
        + u.x * u.y * u.z * (-va + vb + vc - vd + ve - vf - vg + vh);
}

// Tileable 3D worley noise
fn worley_noise(uv: Vec3) -> f32 {
    let id = uv.floor();

    let p = uv.fract();

    let mut min_dist = 10000_f32;
    for x in [-1., 0., 1.] {
        for y in [-1., 0., 1.] {
            for z in [-1., 0., 1.] {
                let offset = vec3(x, y, z);
                let mut h = hash33(id + offset) * 0.5 + 0.5;
                h += offset;
                let d = p - h;
                min_dist = min_dist.min(d.dot(d));
            }
        }
    }

    // inverted worley noise
    return 1. - min_dist;
}

// Fbm for Perlin noise based on iq's blog
fn perlinfbm(mut p: Vec3, fre: f32, octaves: i32) -> f32 {
    let mut freq = fre;
    let g = 2_f32.powf(-0.85);
    let mut amp = 1.;
    let mut noise = 0.;
    for _ in 0..octaves {
        noise += amp * gradient_noise(p * freq, freq);
        freq *= 2.;
        amp *= g;
        p = ROTATE * p;
    }

    return noise;
}

// Tileable Worley fbm inspired by Andrew Schneider's Real-Time Volumetric Cloudscapes
// chapter in GPU Pro 7.
fn worley_fbm(p: Vec3, freq: f32) -> f32 {
    return worley_noise(p * freq) * 0.625
        + worley_noise(ROTATE * p * freq * 2.) * 0.25
        + worley_noise(ROTATE * ROTATE * p * freq * 4.) * 0.125;
}

fn noise_helper(p: Vec3) -> Vec4 {
    let mut col = vec4(0., 0., 0., 0.);

    let freq = 4.;

    let mut pfbm = mix(1., perlinfbm(p, 4., 7), 0.5);
    pfbm = (pfbm * 2. - 1.).abs(); // billowy perlin noise

    col.y += worley_fbm(p, freq);
    col.z += worley_fbm(ROTATE * p, freq * 2.);
    col.w += worley_fbm(ROTATE * ROTATE * p, freq * 4.);
    col.x += remap(pfbm, 0., 1., col.y, 1.); // perlin-worley

    return col;
}

fn mix(a: f32, b: f32, t: f32) -> f32 {
    a * (1. - t) + b * t
}

pub fn noise(p: Vec3) -> f32 {
    let warp = gradient_noise(p, 1.);
    let n = noise_helper(p + warp);

    let perlin_worley = n.x;

    // worley fbms with different frequencies
    let worley = n.yzw();
    let wfbm = worley.x * 0.625 + worley.y * 0.125 + worley.z * 0.25;

    // cloud shape modeled after the GPU Pro 7 chapter
    let mut cloud = remap(perlin_worley, wfbm - 1., 1., 0., 1.);
    cloud = remap(cloud, 0.7, 1., 0., 1.); // fake cloud coverage
    return cloud;
}
