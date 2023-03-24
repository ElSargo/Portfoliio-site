struct CustomMaterial {
    color: vec4<f32>,
    camera_position: vec3<f32>,
};

fn Rayleigh(costh: f32) -> f32
{
    return 3.0 / (16.0 * 3.14159265358979323846) * (1.0 + costh * costh);
}

fn rayleigh ( theta: f32,  lambda: f32) -> f32
{
    let pi = 3.1415926535897932384626433;
    let Kr = 0.5 * PI*PI * pow(1.00029*1.00029-1., 2.) / 2.5e+25;
    return Kr * (1. + pow(cos(theta),2.)) / pow(lambda, 4.);
}

fn numericalMieFit(costh: f32) -> f32
{
    // This function was optimized to minimize (delta*delta)/reference in order to capture
    // the low intensity behavior.
    let bestParams = array(
        9.805233e-06,
        6.500000e+01,
        5.500000e+01,
        8.194068e-01,
        1.388198e-01,
        8.370334e+01,
        7.810083e+00,
        2.054747e-03,
        2.600563e-02,
        4.552125e-12,
    );
    
    let p1 = costh + bestParams[3];
    let expValues = exp(vec4(bestParams[1] *costh+bestParams[2], bestParams[5] *p1*p1, bestParams[6] *costh, bestParams[9] *costh));
    let expValWeight = vec4(bestParams[0], bestParams[4], bestParams[7], bestParams[8]);
    return dot(expValues, expValWeight) * 0.25;
}

fn tonemapACES( x: vec3<f32> ) -> vec3<f32>{
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return (x*(a*x+b))/(x*(c*x+d)+e);
}

@group(1) @binding(0)
var<uniform> material: CustomMaterial;
@group(1) @binding(1)
var base_color_texture: texture_3d<vec3<f32>>;
@group(1) @binding(2)
var base_color_sampler: sampler;

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

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let rd = normalize(world_position.xyz - material.camera_position);
    let sun_dir = normalize(vec3(1.,1.,1.));
    

    
    let rddotsund = dot(-rd, sun_dir);
    let dif = dot(normalize(world_normal),sun_dir );
    let p = max(0.0,numericalMieFit(rddotsund ));
    // let col = max(vec3(0.) , vec3(1.,0.9,0.8) * dif * p) + vec3(0.5,0.6,0.7);
    let s = exp(-uv.x*100.);
    let g = s*2.;

    let col = vec3(g) + vec3(0.2,0.225,0.3)*(dot(normalize(world_normal),vec3(0.,1.,0.))*0.05+.95);
    
    return vec4( (col)*1.5, 1. );
    //material.color * textureSample(base_color_texture, base_color_sampler, uv);
}
