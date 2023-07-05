
struct CustomMaterial {
    sun_direction: vec3<f32>,
    camera_position: vec3<f32>,
    time: f32,
    shadow_dist: f32,
    shadow_coef: f32,
    worley_factor: f32,
    value_factor: f32,
    cloud_coef: f32,
    cloud_height: f32,
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
var w_tex: texture_2d<f32>;
@group(1) @binding(2)
var w_sampler: sampler;
@group(1) @binding(3)
var v_tex: texture_2d<f32>;
@group(1) @binding(4)
var v_sampler: sampler;
@group(1) @binding(5)
var w3_tex: texture_3d<f32>;
@group(1) @binding(6)
var w3_sampler: sampler;

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

// fn hash12(p: vec2<f32>) -> f32 {
//     var p3 = fract(vec3(p.x, p.y, p.x) * .1031);
//     p3 += dot(p3, p3.yzx + 33.33);
//     return fract((p3.x + p3.y) * p3.z);
// }

// fn hash13(p3: vec3<f32>) -> f32 {
//     var p3 = fract(p3 * .1031);
//     p3 += dot(p3, p3.zyx + 31.32);
//     return fract((p3.x + p3.y) * p3.z);
// }


fn fast_ne_exp(x: f32) -> f32 {
    var g = x * 0.06 - 1.0; // 1
    g = g * g; // 2
    g = g * g; // 4
    g = g * g; // 8
    return g * g;
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

fn spe(rd: vec3<f32>, nor: vec3<f32>, sun: vec3<f32>) -> f32 {
    let refel = reflect(rd, nor);
    return pow(max(0., dot(refel, sun)), 20.);
}


// fn cloud(uv: vec2<f32>) -> f32 {
//     return (textureSample(w_tex, w_sampler, uv + material.time*0.005).x  - material.worley_factor) * ( textureSample(v_tex, v_sampler, (uv) - material.time * vec2(-0.01,0.01)  ).x - material.value_factor) * material.cloud_coef ;
// }
// fn cloud(uv: vec2<f32>) -> f32 {
//     let w = textureSample(w_tex, w_sampler, uv + material.time * 0.005) - material.worley_factor ;
//     let v = textureSample(v_tex, v_sampler, (uv) - material.time * vec2(-0.01, 0.01)) - material.value_factor ;
//     let p = vec4(v.x, w.y * v.y, w.z * v.z, w.w * v.w) ;
//     return (p.x + p.y * 0.5 + p.z * 0.25 + p.w * 0.125) * material.cloud_coef ;
// }

fn sdSpiral( p: vec2<f32>, w: f32, h: f32 ) -> f32 {
    // base point
    var p = p;
    var w = w;
    var d = length(p);
    // 8 arcs
    for( var i=0; i<8; i++ ) {
        p.x -= w;
        if( p.x<0.0 && p.y>0.0 ) {d = min( d, abs(length(p)-w) ); }
        p.y -= w;
        p = vec2(-p.y,p.x);
        w *= h;
    }
    // tip point
    return min( d, length(p) );
}

fn rot(a: f32) -> mat2x2<f32> {
    let s = sin(a);
    let c = cos(a);
    return mat2x2(c,-s,s,c);
}

fn cloud(p: vec3<f32>) -> f32 {
    let w3 = textureSample(w3_tex,w3_sampler, vec3(10.0,10.,10.) * (p  + vec3(1.0,0.0,1.0) * material.time * 0.01)).x ;
    let w = textureSample(w_tex, w_sampler, p.xz + material.time * 0.01) - material.worley_factor ;
    let v = textureSample(v_tex, v_sampler, (p.xz* 2.0) - material.time * vec2(0.011, 0.0098)) - material.value_factor ;
    return w3*0.0*v.x  +   pow(w.x, 2.5) *  material.cloud_coef ;// * v.x ;

}



@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let sun_dir = material.sun_direction  ;
    var water = vec3(0.);
    var p = vec3(uv.x,2.0,uv.y);
    var count = 0.;
    var pr = 1.0;
    var pj = 1.0;
    let rd = normalize(world_position.xyz - material.camera_position);
    let dj = 0.01;
    var jp = dj;
    for (var j = 0.1; j < 2.2 ; j += max(0.01,dj * abs(jp ) * j * 100.  )){
        p = vec3(uv.x,2.0,uv.y) + vec3(rd.x,rd.y,-rd.z) *  j * 0.1;
        let samp = max(material.cloud_height,cloud(p));
        jp = (2.0 - j  - samp );
        if 0.0 > jp{
            // p = uv + vec2(rd.x,-rd.z) * 0.1 * j - dj + dj*(pr - pj) / ( (2.0 - j) - pr - pj );
            // count = samp - (2.0 - j);
            break;
        }
        pr = samp;
        pj = 2.0 - j;
        count += 0.2;
    }
    let samp = cloud(p);
    let samps = cloud(p + sun_dir * 0.001);
    let sampd = pow(samps - samp, 1.) ;
    var h = samp;
    let w = cloud((p - 0.3));
    var sha = vec3(1.0);
    let minh = material.cloud_height  ;
    let dens = smoothstep(minh, minh + 0.1, samp);
    if dens == 0.0 {
        h = minh;
        // p  -= sun_dir * 0.01;
    }
    for (var d = 0.1; d < material.shadow_dist; d += d) {
        let s = cloud(p - sun_dir * d * 0.001);
        let u = s - sun_dir.y * d * 0.001 - h;
        sha *= smoothstep(0.1 , -0.004 * d, u * (material.shadow_dist - d) * material.shadow_coef);
    }
    sha = pow(sha, vec3(1.0,0.95,0.9));
    var light = 0.4 *vec3(0.3, 0.5, 1.) * (0.5 + smoothstep(0.05, -0.03, sampd )) + smoothstep(-0.04, 0.04, sampd) * 6.0 * vec3(1.1, 0.8, 0.6) * sha      ;
    // light +=  pow(count ,3.0)* vec3(1.0,1.2,1.5) *  20.1 * (1.0 - sha)  ;
    


        {

        let pos = world_position.xyz * 0.2;
        let rd = normalize(world_position.xyz - material.camera_position);
        let sun = sun_dir * vec3(-1., 1., 1.);
        var noi = value_fbm(pos*2.25 + material.time * 0.3) + value_fbm(pos*1.5 - material.time * 0.3);
        var nor = normalize(mix(noi.yzw, vec3(0., 1., 0.), 0.6));
        let fre = pow(sqrt(
            0.5 + dot(nor, rd) * .5 + .5,
        ), 2.);

        let sdn = dot(nor, sun);
        let deep = vec3(0.05, 0.05, 0.1);
        water = deep + (spe(rd, nor, sun) + max(0., sdn) * vec3(0.0, 0.3, 0.2) * 0.1) * sha ;
    }


    let col = mix(light, water * (1.0 + sha), 1. - dens);
    return vec4(col, 1.0);
    // return vec4(vec3(smoothstep(0.,10., textureSample(w3_tex,w3_sampler,vec3(uv.x,material.time*0.01, uv.y) * 10.0).x ) ) , 1.0);
}
