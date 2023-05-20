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

fn spe(rd: vec3<f32>, nor: vec3<f32>, sun: vec3<f32>) -> f32 {
    let refel = reflect(rd, nor);
    return pow(max(0., dot(refel, sun)), 200.);
}

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let pos = world_position.xyz;
    let rd = normalize(world_position.xyz - material.camera_position);

    let sun_dir = normalize(material.sun_direction * vec3(-1.,-1.,1.));
    var noi = value_fbm(pos * vec3(0.01,0.01,0.005) + vec3(material.time * .5, material.time * 0.1, 0.)) + value_fbm(pos * vec3(0.005,0.01,0.01) + vec3(0., material.time * 0.3, material.time * 0.4));
    var nor = normalize(mix(noi.yzw, vec3(0., 1., 0.), 0.6));
    let fre = pow(sqrt(
        0.5+dot(nor,rd)*.5 + .5,
    ),5.);

    let sdn = dot(nor, sun_dir);
    let deep = vec3(0.075, 0.075, 0.14);
    let shallow = vec3(0.1, 0.5, 0.4);
    let col = deep + spe(rd, nor, sun_dir) + 0.1 * max(0., sdn) ;
    let opa = smoothstep(5000.,4000.,distance(
        material.camera_position.xz,
        world_position.xz
    ));

    return vec4(col, 1.);
}
