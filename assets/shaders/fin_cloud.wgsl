
struct CustomMaterial {
    sun_direction: vec3<f32>,
    camera_position: vec3<f32>,
};

@group(1) @binding(0)
var<uniform> material: CustomMaterial;

@group(1) @binding(1)
var noise_texture: texture_2d<f32>;

@group(1) @binding(2)
var noise_sampler: sampler;

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

fn hash(p: vec3<f32>) -> f32 {
    // replace this by something better {
    var p = fract(p * 0.3183099 + 0.1);
    p *= 17.0;
    return fract(p.x * p.y * p.z * (p.x + p.y + p.z));
}

fn value_noise(x: vec3<f32>) -> vec4<f32> {
    let i = floor(x);
    let w = fract(x);

    let u = w * w * w * (w * (w * 6.0 - 15.0) + 10.0);
    let du = 30.0 * w * w * (w * (w - 2.0) + 1.0);    let a = hash(i + vec3(0.0, 0.0, 0.0));
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

    let deriv = du * vec3(
        k1 + k4 * u.y + k6 * u.z + k7 * u.y * u.z,
        k2 + k5 * u.z + k4 * u.x + k7 * u.z * u.x,
        k3 + k6 * u.x + k5 * u.y + k7 * u.x * u.y,
    );
    return vec4(
        k0 + k1 * u.x + k2 * u.y + k3 * u.z + k4 * u.x * u.y + k5 * u.y * u.z + k6 * u.z * u.x + k7 * u.x * u.y * u.z,
        deriv.x,
        deriv.y,
        deriv.z,
    );
}

fn value_fbm(p: vec3<f32>) -> vec4<f32 > {
    var p = p ;
    var t = vec4(0.);
    var s = 1.;
    var c = 1.;

    for (var i = 0; i < 3 ; i++) {
        p += vec3(13.123, -72., 234.23);
        t += (value_noise(p * s)) * c;
        s *= 2.;
        c *= 0.5;
    }
    return t / 2.7;
}



@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let pos = world_position.xyz;
    let sun_dir = vec3(0.,1.,0.);//normalize(material.sun_direction * -1.);
    let rd = normalize(world_position.xyz - material.camera_position);
    let nor = normalize(world_normal.xyz);
    let u = textureSample(noise_texture, noise_sampler, uv).xyz;
    // let u = textureSampleBaseClampToEdge(noise_texture,noise_sampler,uv).xy;
    let me = mie(dot(rd, sun_dir)) + 0.5;
    let dens = uv.x;
    let sha = smoothstep(0.3, 1., u.y);
    var opa = 1.;
    if uv.x > 0.5 {
        opa = smoothstep(1., 0.4, uv.x) ;//* smoothstep(0.7, 1., abs(dot(rd, nor)));
    }       
    // opa= mix(u.y*0.001, opa,opa)*3.;
    opa = max(0., opa - u.x * smoothstep(1., 0., opa));

    return vec4(vec3(1.+me*9.) * smoothstep(0., 2., u.z) * 3. + 0.5, opa * min(1., 1.2 + u.y));
    // return vec4(uv,0.,1.);
    // return vec4(vec3(opa),1.);
}
