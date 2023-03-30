struct CustomMaterial {
    color: vec4<f32>,
    camera_position: vec3<f32>,
    aabb_position: vec3<f32>,
    texture_dim: vec3<f32>,
    scale: vec3<f32>,
    time: f32,
};

fn Rayleigh(costh: f32) -> f32 {
    return 3.0 / (16.0 * 3.14159265358979323846) * (1.0 + costh * costh);
}

fn rayleigh(theta: f32, lambda: f32) -> f32 {
    let pi = 3.1415926535897932384626433;
    let Kr = 0.5 * pi * pi * pow(1.00029 * 1.00029 - 1., 2.) / 2.5e+25;
    return Kr * (1. + pow(cos(theta), 2.)) / pow(lambda, 4.);
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
    let a = x * 0.2 - 1.;
    let b = a * a;
    return b * b;
}

fn powder(x: f32) -> f32 {
    let a = x * 0.2 - 1.; // nearly exp(-x)
    let b = a * a; // pow 2
    let c = b * b; // pow 4
    let d = c * c; // pow 8
    let e = d * d; // pow 16
    return c - e * e;
}

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let ro = material.camera_position - material.aabb_position;
    let rd = normalize(world_position.xyz - material.camera_position);
    let sun_dir = normalize(vec3(1., 1., 0.));
    let mei = mie(dot(rd, sun_dir));
    let inv_sca = 1./material.scale;
    let mo = ro +0.5*material.scale;
    var p = ro;
    var dt = 0.1;
    var steps = 0.;
    var light = vec3(0.);
    var absorbtion = 0.;
    let intersection = boxIntersection(ro, rd, vec3(0.5)*material.scale);
    var i = max(intersection.x, 0.) + hash13(vec3(uv * 913.123, material.time )) * .04;
    for (; i < intersection.y; i += dt * 100.) {
        let transmission = powder(absorbtion);
        if absorbtion > 2. {
            absorbtion = 6.;
            break;
        }
        steps += 1.;
        p = mo + i * rd;
        let samp = sdf(p*inv_sca );
        let d = samp.x;
        if d < 0.0 {
            let dens = mix(-d,1.,0.5);
            absorbtion += dt * 10. * dens;
            let direct = (samp.y * (mei+.5) * 300.) * 1.  ;
            let scattered =   3. ;
            light += (direct + scattered ) * transmission * dt  * dens;
        }
        dt = max(samp.x  , 0.01);
    }

    // return vec4( vec3(1. - fast_ne_exp(absorbtion)),1.);
    return vec4(light, 1. - fast_ne_exp(absorbtion));
    // return vec4( vec3(steps*0.01),1.);
    // return vec4(
    //     vec3(sdf(vec3(world_position.xy+0.5,0.5)).x),1.
    // );


}
