struct CubemapMaterial {
    camera_position: vec3<f32>,
    sun_direction: vec3<f32>,
    time: f32,
};


#import bevy_pbr::mesh_view_bindings

@group(1) @binding(0)
var<uniform> material: CubemapMaterial;

@group(1) @binding(1)
var noise_texture: texture_2d<f32>;

@group(1) @binding(2)
var noise_sampler: sampler;

@group(1) @binding(3)
var volume_texture: texture_3d<f32>;

@group(1) @binding(4)
var volume_sampler: sampler;

@group(1) @binding(5)
var base_color_texture: texture_cube<f32>;

@group(1) @binding(6)
var base_color_sampler: sampler;


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

fn rayleigh(costh: f32) -> f32 {
    return 3.0 / (16.0 * 3.14159265358979323846) * (1.0 + costh * costh);
}

fn fre(cos_theta_incident: f32) -> f32 {
    let p = 1.0 - cos_theta_incident;
    let p2 = p * p;
    return p2 * p2 * p;
}

fn fnexp(x: f32) -> f32 {
    let a = 0.2 * x + 1.;
    let b = a * a;
    return b * b;
}

fn fnexp3(x: vec3<f32>) -> vec3<f32> {
    let a = 0.2 * min(x, vec3(6.)) + 1.;
    let b = a * a;
    return b * b;
}

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let fragment_position_view_lh = world_position.xyz * vec3<f32>(1.0, 1.0, -1.0);
    var rd = normalize(world_position.xyz - material.camera_position);
    let sun = normalize(material.sun_direction * vec3(-1., -1., 1.));
    var water_mul = vec3(1.);


    if (rd.y < 0.){
        water_mul = vec3(0.05);
        rd.y = -rd.y ;
    }

    let d = sqrt(
        1. + 12. * (1.-(1. - rd.y) * (1. - rd.y))
    );
    let rds = dot(rd, sun);
    let phase = rayleigh(rds);
    let mie_phase = mie(rds);
    var glow = exp(-d * vec3(4., 2., 1.) * .35)*2.;
    glow +=  exp(-d * vec3(1., 2., 4.) * 1.3 + mie_phase * 1.);
    glow += smoothstep(0.99,1.01,rds)+10.*smoothstep(0.999,1.0,rds)*vec3(1.,0.9,0.7);
    return vec4(max(vec3(0.),glow*4.*water_mul), 1.);
}
