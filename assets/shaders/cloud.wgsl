
struct CustomMaterial {
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



fn rot(a: f32) -> mat2x2<f32> {
    let s = sin(a);
    let c = cos(a);
    return mat2x2(c,-s,s,c);
}

fn cloud(p: vec3<f32>) -> f32 {
    // let h =  2.0*(abs(length(p.xz - 0.5) - 0.4) - 0.05) ;
    // let h = p.x - 0.5 + sin(p.z*3. + material.time * 1.1)*1.;
    let w = textureSample(w_tex, w_sampler, p.xz - material.time*0.01).x - material.worley_factor ;
    let x = textureSample(w_tex, w_sampler, p.xz + material.time*0.01).x - material.worley_factor ;
    let z = textureSample(v_tex, v_sampler, p.xz + material.time*vec2(0.01,-0.01)).x - material.value_factor ;
    // return z*(w*smoothstep(0.3,-0.3,h) )*material.cloud_coef ;
    return z*(1.+w)*material.cloud_coef ;
}



@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let sun_dir = material.sun_direction  ;
    var water = vec3(0.);
    var p = vec3(uv.x,2.0,uv.y);
    // var count = 0.;
    // var pr = 1.0;
    // var pj = 1.0;
    // let rd = normalize(world_position.xyz - material.camera_position);
    // let dj = 0.01;
    // var jp = dj;
    // for (var j = 0.1; j < 2.2 ; j += max(0.01,dj * abs(jp ) *  75.  )){
    //     p = vec3(uv.x,2.0,uv.y) + vec3(rd.x,rd.y,-rd.z) *  j * 0.1;
    //     let samp = max(material.cloud_height,cloud(p));
    //     jp = (2.0 - j  - samp );
    //     if 0.0 > jp{
    //         // p = uv + vec2(rd.x,-rd.z) * 0.1 * j - dj + dj*(pr - pj) / ( (2.0 - j) - pr - pj );
    //         // count = samp - (2.0 - j);
    //         break;
    //     }
    //     pr = samp;
    //     pj = 2.0 - j;
    //     count += 1.;
    // }
    let samp = cloud(p);
    let samps = cloud(p + sun_dir * 0.001);
    let sampd = pow(samps - samp, 1.) ;
    var h = samp;
    let w = cloud((p - 0.3));
    var sha = vec3(1.0);
    let minh = material.cloud_height  ;
    let dens = smoothstep(minh , minh + 0.1, samp);
    var maxh = h;
    if dens == 0.0 {
        h = minh;
        // p  -= sun_dir * 0.01;
    }
    for (var d = 0.1; d < material.shadow_dist; d += d) {
        let s = cloud(p - sun_dir * d * 0.001);
        maxh = max(maxh,s);
        let u = s - sun_dir.y * d * 0.001 - h;
        sha *= smoothstep(0.1 , -0.004 * d, u * (material.shadow_dist - d) * material.shadow_coef);
    }
    // sha = pow(sha, vec3(0.4, 0.41,0.5));
    var light = vec3(0.);
    light += 0.4 *vec3(0.3, 0.5, 1.) * (0.5 + smoothstep(0.05, -0.03, sampd )) + smoothstep(-0.04, 0.04, sampd) * 2.0 * vec3(2., 1.5, 1.) * sha ;
    light += 1.0*vec3(0.7,.6,.4)*exp(material.sun_pen * (samp - maxh) )  ;
    // light -= vec3(1.,0.75,0.5) * max(0.,(1.- exp(0.05*(-count + 5.0))));
    


        {
        sha = mix(sha,pow(sha,vec3(0.3)),1. - dens);

        let pos = world_position.xyz * vec3(0.002,0.002,0.002);
        let posd = (material.sun_direction* vec3(1.,1.,-1.)*0.00001 + world_position.xyz * vec3(0.002,0.002,0.002));
        let rd = normalize(world_position.xyz - material.camera_position);
        let sun = sun_dir * vec3(-1., 1., 1.);
        let noi = 2.0 - 1.5 * abs(textureSample(v_tex, v_sampler, (pos.xz +  material.time * 0.01))*textureSample(v_tex, v_sampler, (pos.xz -  material.time * 0.01 + 0.23123)) - 0.1 ) ;
        let noid = 2.0 - 1.5 * abs(textureSample(v_tex, v_sampler, (posd.xz +  material.time * 0.01))*textureSample(v_tex, v_sampler, (posd.xz -  material.time * 0.01 + 0.23123)) - 0.1 ) ;
        let s = (noi - noid) * 1000.;
        let shine = pow(max(0.0,dot(rd,sun)*s.x),2.5)*sha ;
        water = 1000.0*shine+ vec3(0.01,0.02,0.1) + 1.5*vec3(0.06,0.15,0.12)*smoothstep(-0.4,1.,-s.x)*max(0.,-noi.x + 2.5);
    }


    let col = mix(light , water * (1.0 + sha), 1. - dens);
    let at = exp(-vec3(4.0,2.0,1.0)  * 0.00015 * (10. +  length(world_position.xz)));
    let ap = exp(-vec3(1.0,2.0,4.0)  * 0.00015 * (10. +  length(world_position.xz)));
    return vec4(  at*col + (1.0 - ap) , 1.0);
    // return vec4(mix(vec3(0.2,1.,0.2), vec3(1.0,0.0,0.), count * 0.1),1.0);
    // return vec4(vec3(cloud(world_position.xyz * 0.001)*0.2),1.0);
}
