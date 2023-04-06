struct CustomMaterial {
    sun_direction: vec3<f32>,
    camera_position: vec3<f32>,
    scale: vec3<f32>,
    time: f32,
};

fn rayleigh(costh: f32) -> f32 {
    return 3.0 / (16.0 * 3.14159265358979323846) * (1.0 + costh * costh);
}

fn HenyeyGreenstein(g: f32, costh: f32) -> f32 {
    let pi = 3.1415926535897932384626433;
    return (1.0 - g * g) / (4.0 * pi * pow(1.0 + g * g - 2.0 * g * costh, 1.5));
}


fn mie(costh: f32) -> f32 {
    // This function was optimized to minimize (delta*delta)/reference in order to capture
    // the low intensity behavior.
    let params = array(
        9.805233e-06,
        -6.500000e+01,
        -5.500000e+01,
        8.194068e-01,
        1.388198e-01,
        -8.370334e+01,
        7.810083e+00,
        2.054747e-03,
        2.600563e-02,
        -4.552125e-12
    );

    let p1 = costh + params[3];
    let expValues: vec4<f32> = exp(vec4(params[1] * costh + params[2], params[5] * p1 * p1, params[6] * costh, params[9] * costh));
    let expValWeight: vec4<f32> = vec4(params[0], params[4], params[7], params[8]);
    return dot(expValues, expValWeight) * 0.25;
}


@group(1) @binding(0)
var<uniform> material: CustomMaterial;

@group(1) @binding(1)
var noise_texture: texture_3d<f32>;

@group(1) @binding(2)
var noise_sampler: sampler;

// @location(0) world_position: vec4<f32>,
// @location(1) world_normal: vec3<f32>,
// #ifdef VERTEX_UVS
// @location(2) uv: vec2<f32>,
// #endif
// #ifdef VERTEX_TANGENTS
// @location(3) world_tangent: vec4<f32>,
// #endif
// #ifdef VERTEX_COLORS
// @location(4) color: vec4<f32>,
// #endif

fn boxIntersection(ro: vec3<f32>, rd: vec3<f32>, boxSize: vec3<f32>) -> vec2<f32> {
    let m = 1.0 / rd; // can precompute if traversing a set of aligned boxes
    let n = m * ro;   // can precompute if traversing a set of aligned boxes
    let k = abs(m) * boxSize;
    let t1 = -n - k;
    let t2 = -n + k;
    let tN = max(max(t1.x, t1.y), t1.z);
    let tF = min(min(t2.x, t2.y), t2.z);
    if tN > tF || tF < 0.0 {return vec2(-1.0);}; // no intersection
    return vec2(tN, tF);
}

fn hash12(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3(p.x, p.y, p.x) * .1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn hash13(p3: vec3<f32>) -> f32 {
    var p3 = fract(p3 * .1031);
    p3 += dot(p3, p3.zyx + 31.32);
    return fract((p3.x + p3.y) * p3.z);
}

fn fast_ne_exp(x: f32) -> f32 {
    let a = x * 0.2 - 1.;
    let b = a * a;
    return b * b;
}
fn hash(p: vec3<f32>) -> f32 {// replace this by something better {
    var p = fract(p * 0.3183099 + .1);
    p *= 17.0;
    return fract(p.x * p.y * p.z * (p.x + p.y + p.z));
}

fn noised(x: vec3<f32>) -> vec4<f32> {
    let i = floor(x);
    let w = fract(x);
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
    let k7 = - a + b + c - d + e - f - g + h;

    return vec4(k0 + k1 * u.x + k2 * u.y + k3 * u.z + k4 * u.x * u.y + k5 * u.y * u.z + k6 * u.z * u.x + k7 * u.x * u.y * u.z, du * vec3(k1 + k4 * u.y + k6 * u.z + k7 * u.y * u.z, k2 + k5 * u.z + k4 * u.x + k7 * u.z * u.x, k3 + k6 * u.x + k5 * u.y + k7 * u.x * u.y));
}



fn fbmd(p: vec3<f32>) -> vec4<f32> {
    var p = p + vec3(0., material.time * 0.1, 0.);
    var T = vec4(0.);
    var s = 1.;
    var c = 1.;

    for (var i = 0; i < 6; i++) {
        p += vec3(13.123, -72., 234.23);
        let n = noised(p * s) * c;
        T.x += n.x;
        if i < 1 {
            T.y += n.y;
            T.z += n.z;
            T.w += n.w;
        }
        s *= 2.;
        c *= 0.65;
    }
    return T / 2.7;
}

fn hash33(p3: vec3<f32>) -> vec3<f32> {
    var p3 = fract(p3 * vec3(.1031, .1030, .0973));
    p3 += dot(p3, p3.yxz + 33.33);
    return fract((p3.xxy + p3.yxx) * p3.zyx);
}


fn worley_noise(uv: vec3<f32>) -> f32 {
    let id = floor(uv);

    let p = fract(uv);

    var min_dist = 10000.;
    for (var x = -1.; x < 1.5; x += 1.) {
        for (var y = -1.; y < 1.5; y += 1.) {
            for (var z = -1.; z < 1.5; z += 1.) {
                let offset = vec3(x, y, z);
                var h = hash33(id + offset) * 0.5 + 0.5;
                h += offset;
                let d = p - h;
                min_dist = min(min_dist, dot(d,d));
            }
        }
    }

    // inverted worley noise
    return min_dist;
}


fn wfbm(p: vec3<f32>) -> f32{
    var p = p;
    var T = 0.0;
    var s = 1.;
    var c = 1.;

    for (var i = 0; i < 5; i++) {
        p += vec3(13.123, -72., 234.23);
        let n = worley_noise(p * s,  ) ;
        T += n * c ;
        s *= 2.;
        c *= 0.5;
    }
    // T.x = max(0., T.x);
    return T  ;
    
}

fn powder(x: f32) -> f32 {
    let a = x * 0.2 - 1.; // nearly exp(-x)
    let b = a * a; // 
    let c = b * b; // Base
    let d = c * c; // pow 2
    let e = d * d; // pow 4
    let f = e * e; // pow 8
    return d - f * f;
}


fn psfbm(x: vec3<f32>) -> vec4<f32> {

    var p = x;
    var T = vec4(0.);
    var s = 1.;
    var c = 1.;

    for (var i = 0; i < 3; i++) {
        // p += vec3(13.123, -72., 234.23);
        let n = psrdnoise3(p * s, vec3(100.), 0. * material.time) ;
        T.x += abs(n.noise) * c ;
        T.y += n.gradient.x * c;
        T.z += n.gradient.y * c;
        T.w += n.gradient.z * c;
        s *= 2.;
        c *= 0.5;
    }
    // T.x = max(0., T.x);
    return T  ;
}

fn reflVec(v: vec3<f32>, r: vec3<f32>) -> vec3<f32> {
    let k = dot(v, r);
    if k > 0.0 {
        return v  ;
    } else {
        return v - 2.0 * r * k;
    }
}

#import "shaders/psrdnoise.wgsl"

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let rd = normalize(world_position.xyz - material.camera_position);
    let sample_position = world_position.xyz / material.scale * 6. ;

    let wnoi = wfbm(0.5*sample_position);
    let wnoid = wfbm(0.5*sample_position+material.sun_direction);
    var noi = fbmd(sample_position);
    noi.x = mix( noi.x ,wnoi, 0.4);
    // let noi = vec4(
    //     np.noise,
    //     np.gradient
    // );
    var geo_nor = normalize(world_normal.xyz);
    let sun_dir = normalize(material.sun_direction * -1.);
    let me = mie(dot(rd, sun_dir));
    let scl = vec3(1., 1., 1.);
    let sac = vec3(0.3);
    let shallow = abs(dot(rd, geo_nor));
    // let powder = 1.-smoothstep(0.6,-1.,dot(nor,noi.yzw));
    var opa = smoothstep(0.4, 0.8, shallow);
    opa -= (noi.x * noi.x * noi.x) * smoothstep(1., 0.4, shallow) * 30.;
    let nor = normalize(geo_nor + reflVec(normalize(noi.xyz), geo_nor));
    let col = mix(
        scl * 3. + me * 20.,
        sac * (2. + me * 8.),
        smoothstep(1., -.6, dot(sun_dir, nor) + noi.x * 0.4 + 0.3*(wnoid - wnoi)) - 0.2*smoothstep(0.99,0.9,shallow  )
    )  ;

    // let fade = exp(vec3(2., 2., 2.) * distance(world_position.xyz, material.camera_position) * -0.00015);

    return vec4(col * opa, max(0., min(1., opa - 1. + exp(-0.0003 * distance(world_position.xyz, material.camera_position)))));
}
