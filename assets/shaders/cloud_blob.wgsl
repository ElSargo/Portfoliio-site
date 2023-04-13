struct CustomMaterial {
    sun_direction: vec3<f32>,
    camera_position: vec3<f32>,
    scale: vec3<f32>,
    time: f32,
};

fn fast_ne_exp(x: f32) -> f32 {
    let a = x * 0.2 - 1.;
    let b = a * a;
    return b * b;
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

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let rd = normalize(world_position.xyz - material.camera_position);
    var sample_position = world_position.xyz * 0.007   ;

    let noi = textureSample(noise_texture, noise_sampler, abs(fract(0.12 * sample_position) - 0.5) * 2.).x;
    let noid = textureSample(noise_texture, noise_sampler, abs(fract(0.12 * (sample_position + material.sun_direction * .40)) - 0.5) * 2.).x;

    let nor = normalize(world_normal.xyz);
    let sun_dir = normalize(material.sun_direction * -1.);
    // let me = mix(mie(dot(rd, sun_dir)), 1., 0.25);
    let me = mie(dot(rd, sun_dir)) ;
    let scl = vec3(1., 1., 1.);
    let sac = vec3(1., 0.9, 0.8);
    let shallow = abs(dot(rd, nor));
    var opa = smoothstep(0.4, 0.8, shallow);
    opa -= (noi * noi * noi) * smoothstep(1., 0.4, shallow) * 20.;
    let derdirv = (noi - noid);
    let col = mix(
        scl + me * 2.,
        sac + 0.5 + me * 0.6,
        min(smoothstep(0.5, -0.5, dot(sun_dir, nor)), smoothstep(0.7, 0.99, shallow))
        // min(1., smoothstep(-1., 1., dot(sun_dir, nor)  ) - 0.0 * smoothstep(0.99, 0.9, shallow - (noi - 0.5) * 0.1))
    ) - derdirv *  vec3(1.5,1.2,0.9);

    let p = max(0., min(1., opa));
    return vec4(col, p);
}
