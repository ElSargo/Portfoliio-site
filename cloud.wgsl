
struct CustomMaterial {
    color: vec4<f32>,
    camera_position: vec3<f32>,
    aabb_position: vec3<f32>,
    texture_dim: vec3<f32>,
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
var volume_tex: texture_3d<f32>;
@group(1) @binding(2)
var volume_sampler: sampler;

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

fn sdf(p: vec3<f32>) -> vec4<f32> {
    return textureSample(volume_tex, volume_sampler, p);
}

fn fast_ne_exp(x: f32) -> f32 {
    var g = x * 0.06 - 1.0; // 1
    g = g * g; // 2
    g = g * g; // 4
    g = g * g; // 8
    return g * g;
}

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let ro = material.camera_position - material.aabb_position;
    let distance = distance(ro, world_position.xzy);
    let rd = normalize(world_position.xyz - material.camera_position);
    let sun_dir = normalize(vec3(cos(material.time), .3, sin(material.time)));
    let mei = mie(dot(rd, sun_dir));
    let inv_sca = 1. / material.scale;
    let mo = ro + 0.5 * material.scale;
    var light = vec3(1.);
    let intersection = boxIntersection(ro, rd, vec3(0.5) * material.scale);
    let samp = sdf((mo + rd * intersection.x) * inv_sca);
    let samps = sdf((mo + rd * intersection.x + sun_dir) * inv_sca);
    let h = samp.x;
    var sha = vec3(1.0);
    for (var d = 0.1; d < 100.0; d += 1.1) {
        let s = sdf((mo + rd * intersection.x - sun_dir * d) * inv_sca);
        let u = s.x - sun_dir.y * d * 0.001 - h;
        sha *= smoothstep(0.1, 0., u / d * 5.1);
    }
    sha = pow(sha, vec3(0.8, 0.9, 1.));
    let sampd = (samps - samp) ;
    let dens = smoothstep(0.6, 0.7, samp.x);
    light = vec3(0.3, 0.5, 1.) * smoothstep(-0.08, 0.1, -sampd.x) + smoothstep(-0.02, 0.04, sampd.x) * 6.0 * vec3(1.1, 0.8, 0.6) * sha + 0.1   
    ;
    return vec4(light  , 1. - fast_ne_exp(dens));
}
