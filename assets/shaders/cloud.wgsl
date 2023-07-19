
struct Material {
    sun_direction: vec3<f32>,
    camera_position: vec3<f32>,
    time: f32,
    shadow_dist: f32,
    shadow_coef: f32,
    sun_pen: f32,
    worley_factor: f32,
    value_factor: f32,
    cloud_coef: f32,
    cloud_height: f32,
    scroll: f32,
};

@group(1) @binding(0)
var<uniform> material: Material;
@group(1) @binding(1)
var w_tex: texture_2d<f32>;
@group(1) @binding(2)
var w_sampler: sampler;
@group(1) @binding(3)
var v_tex: texture_2d<f32>;
@group(1) @binding(4)
var v_sampler: sampler;

fn step(a: f32, b: f32, t: f32) -> f32 {
    let x = t - a;
    let y = x / (b - a);
    return clamp(0., 1., y);
}

fn sabs(x: f32, j: f32) -> f32 {
    return sqrt(x * x + j);
}

fn cloud(p: vec2<f32>) -> f32 {
    let g = sabs(length(p) - 0.8 + sin(material.time) * 0.2, 0.001) + 0.7 ;
    let w = (textureSample(w_tex, w_sampler, p - material.time * 0.01).x) - material.worley_factor  ;
    let z = textureSample(v_tex, v_sampler, p + material.time * vec2(0.01, -0.01)).x - material.value_factor ;
    return z * (1. + w) * material.cloud_coef - step(0.8, 1.6, g)   ;
}



@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let sun_dir = material.sun_direction  ;
    var p = uv + material.scroll * vec2(0., 1.0);
    let samp = cloud(p);
    let samps = cloud(p + sun_dir.xz * 0.001);
    let sampd = samps - samp;
    var h = samp;
    var sha = vec3(1.0);
    let minh = material.cloud_height ;
    let dens = smoothstep(minh, minh + 0.02, samp);
    var maxh = h;
    var shap = vec3(1.0);
    if dens <= 0.1 {
        h = minh;
        p -= sun_dir.xz * 0.02  ;
    }
    for (var d = 0.1; d < material.shadow_dist; d += d) {
        let s = cloud(p - sun_dir.xz * d * 0.001);
        maxh = max(maxh, s);
        let u = s - sun_dir.y * d * 0.001 - h;
        sha *= smoothstep(0.1, -0.004 * d, u * (material.shadow_dist - d) * material.shadow_coef) ;
    }
    sha = pow(sha, shap);
    var light = vec3(0.);
    light += 0.4 * vec3(0.3, 0.5, 1.) * (0.5 + smoothstep(0.05, -0.03, sampd)) + smoothstep(-0.04, 0.04, sampd) * 2.0 * vec3(2., 1.5, 1.) * sha ;
    light += vec3(0.7, .6, .4) * exp(material.sun_pen * (samp - maxh))  ;



    var water = vec3(0.);
        {// Water
        sha = mix(sha, pow(sha, vec3(0.4)), 1. - dens);
        // sha *= smoothstep(0.2,0.1,dens);
        p *= 4.;
        let pd = (sun_dir.xz * 0.00001 + p);
        let rd = normalize(world_position.xyz - material.camera_position);
        let sun = sun_dir * vec3(-1., 1., 1.);
        let noi = 2.0 - 1.5 * abs(textureSample(v_tex, v_sampler, (p + material.time * 0.02)) * textureSample(v_tex, v_sampler, (p - material.time * 0.02 + vec2(1.123, 1.33123))) - 0.1) ;
        let noid = 2.0 - 1.5 * abs(textureSample(v_tex, v_sampler, (pd + material.time * 0.02)) * textureSample(v_tex, v_sampler, (pd - material.time * 0.02 + vec2(1.123, 1.33123))) - 0.1) ;
        let s = (noi - noid) * 1000.;
        let shine = pow(max(0.0, dot(rd, sun) * 0.03 + s.x * 0.5), 2.5) * sha ;
        water = 1000.0 * shine + vec3(0.01, 0.02, 0.1) + 1.5 * vec3(0.06, 0.15, 0.12) * smoothstep(-0.4, 1., -s.x) * max(0., -noi.x + 2.5);
    }


    let col = mix(light, water * (1.0 + sha), 1. - dens);
    let at = exp(-vec3(4.0, 2.0, 1.0) * 0.00015 * (10. + length(world_position.xz)));
    let ap = exp(-vec3(1.0, 2.0, 4.0) * 0.00015 * (10. + length(world_position.xz)));
    return vec4(at * col + (1.0 - ap), 1.0);
}
