struct CustomMaterial {
    sun_direction: vec3<f32>,
    camera_position: vec3<f32>,
    scale: vec3<f32>,
    time: f32,
};

@group(1) @binding(0)
var<uniform> material: CustomMaterial;
@group(1) @binding(1)
var noise_texture: texture_3d<f32>;
@group(1) @binding(2)
var noise_sampler: sampler;


fn mie(costh: f32) -> f32 {
    let params = array(9.805233e-06, -6.500000e+01, -5.500000e+01, 8.194068e-01, 1.388198e-01, -8.370334e+01, 7.810083e+00, 2.054747e-03, 2.600563e-02, -4.552125e-12);
    let p1 = costh + params[3];
    let expValues: vec4<f32> = exp(vec4(params[1] * costh + params[2], params[5] * p1 * p1, params[6] * costh, params[9] * costh));
    let expValWeight: vec4<f32> = vec4(params[0], params[4], params[7], params[8]);
    return dot(expValues, expValWeight) * 0.25;
}

fn almost_identity(x: f32, m: f32, n: f32) -> f32 {
    if x > m {return x;}
    let a = 2.0 * n - m;
    let b = 2.0 * m - 3.0 * n;
    let t = x / m;
    return (a * t + b) * t * t + n;
}

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let ray_direction = normalize(world_position.xyz - material.camera_position);
    var sample_position = world_position.xyz * 0.009   ;
    let normal = normalize(world_normal.xyz);
    let noise = textureSample(noise_texture, noise_sampler, abs(fract(0.12 * sample_position) - 0.5) * 2.).x;
    let dxnoise = textureSample(noise_texture, noise_sampler, abs(fract(0.12 * (sample_position - vec3(0., 10., 0.) - ray_direction * 3.)) - 0.5) * 2.).x;
    let sun_dir = normalize(material.sun_direction * vec3(-1., -1., 1.));
    let mie_signal = mie(dot(ray_direction, sun_dir)) ;
    let sun_color = vec3(1.1, 1.1, 1.) ;
    let shadow_color = vec3(1.0,1.1,1.2)*1.4;
    let shallow = abs(dot(ray_direction, normal));
    let opacity = smoothstep(0.4, 0.8, shallow) - pow((noise * noise * noise) * smoothstep(1., 0.4, shallow) * 20., 2.);
    let directional_derivative = (noise - dxnoise);
    let egde_signal = smoothstep(0.7, 0.99, shallow);
    let blobyness = pow(almost_identity(directional_derivative * egde_signal, .1, 0.05), .4);
    let color = mix(
        (sun_color + blobyness * 4. + mie_signal*sun_color) * (0.6 + smoothstep(-1., -0., normal.y)),
        (shadow_color + blobyness * vec3(0.5, 0.6, 0.7) - mie_signal * 0.2),
        min(smoothstep(1., -1., dot(sun_dir, normal)), egde_signal)
    );

    let clamped_opacity = max(0., min(1., opacity))  ;
    return vec4(smoothstep(vec3(0.0), vec3(5., 5., 5.) - mie_signal, color) * 3. * clamped_opacity, clamped_opacity);
}
