struct NoiseMaterial {
    octaves: i32,
    scale: f32,
    contribution: f32,
    
    falloff: f32,
    threshold: f32,
 };

@group(1) @binding(0)
var<uniform> i: NoiseMaterial ;
@group(1) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(1) @binding(2)
var base_color_sampler: sampler;

// #define_import_path bevy_pbr::mesh_vertex_output

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

fn hash(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3(p.xyx) * .1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}
   
//  fn noise(p: vec2<f32>) -> f32 {
//     let cell = floor(p);
//     let tr = cell + vec2(1., 1.);
//     let tl = cell + vec2(0., 1.);
//     let br = cell + vec2(1., 0.);
//     let bl = cell + vec2(0., 0.);
//     let htr = hash(tr);
//     let htl = hash(tl);
//     let hbr = hash(br);
//     let hbl = hash(bl);
//     let bx = smoothstep(0., 1., fract(p.x));
//     let by = smoothstep(0., 1., fract(p.y));
//     let mt = mix(htl, htr, bx);
//     let mb = mix(hbl, hbr, bx);

//     return mix(mb, mt, by);
// }

// fn clip(a: f32, t: f32) -> f32 {
//     return (a-t) / (1.-t);
// }

// fn fbm(p: vec2<f32>) -> f32 {
//     var l = clip(0.5*(noise(p)+noise(p*2.)), 0.65);
//     l += noise(p*4.)*0.5;    
//     l += noise(p*5.)*0.3;    
//     l += noise(p*7.)*0.2;

//     return l;


// } 

fn hash33(p3: vec3<f32>) -> vec3<f32>
{
    var p = p3;
	p = fract(p * vec3(.1031, .1030, .0973));
    p += dot(p, p.yxz+33.33);
    return fract((p.xxy + p.yxx)*p.zyx);

}
fn remap(x: f32, a: f32, b: f32, c: f32, d: f32) -> f32
{
    return (((x - a) / (b - a)) * (d - c)) + c;
}

// Gradient noise by iq (modified to be tileable)
fn gradientNoise(x: vec3<f32>, freq: f32) -> f32
{
    // grid
    let p = floor(x);
    let w = fract(x);
    
    // quintic interpolant
    let u = w * w * w * (w * (w * 6. - 15.) + 10.);

    
    // gradients
    let ga = hash33(p + vec3(0., 0., 0.) % freq);
    let gb = hash33(p + vec3(1., 0., 0.) % freq);
    let gc = hash33(p + vec3(0., 1., 0.) % freq);
    let gd = hash33(p + vec3(1., 1., 0.) % freq);
    let ge = hash33(p + vec3(0., 0., 1.) % freq);
    let gf = hash33(p + vec3(1., 0., 1.) % freq);
    let gg = hash33(p + vec3(0., 1., 1.) % freq);
    let gh = hash33(p + vec3(1., 1., 1.) % freq);
    
    // projections
    let va = dot(ga, w - vec3(0., 0., 0.));
    let vb = dot(gb, w - vec3(1., 0., 0.));
    let vc = dot(gc, w - vec3(0., 1., 0.));
    let vd = dot(gd, w - vec3(1., 1., 0.));
    let ve = dot(ge, w - vec3(0., 0., 1.));
    let vf = dot(gf, w - vec3(1., 0., 1.));
    let vg = dot(gg, w - vec3(0., 1., 1.));
    let vh = dot(gh, w - vec3(1., 1., 1.));
	
    // interpolation
    return va + 
           u.x * (vb - va) + 
           u.y * (vc - va) + 
           u.z * (ve - va) + 
           u.x * u.y * (va - vb - vc + vd) + 
           u.y * u.z * (va - vc - ve + vg) + 
           u.z * u.x * (va - vb - ve + vf) + 
           u.x * u.y * u.z * (-va + vb + vc - vd + ve - vf - vg + vh);
}

// Tileable 3D worley noise
fn worleyNoise(uv: vec3<f32>, freq: f32) -> f32
{    
    let id = floor(uv);
    let p = fract(uv);
    
    var minDist = 10000.;
    for (var x = -1.; x <= 1.; x += 1.)
    {
        for(var y = -1.; y <= 1.; y += 1.)
        {
            for(var z = -1.; z <= 1.; z += 1.)
            {
                let offset = vec3(x, y, z);
            	var h = hash33(id + offset % vec3(freq)) * .5 + .5;
    			h += offset;
            	let d = p - h;
           		minDist = min(minDist, dot(d, d));
            }
        }
    }
    
    // inverted worley noise
    return 1. - minDist;
}

// Fbm for Perlin noise based on iq's blog
fn perlinfbm(p: vec3<f32>, fre: f32, octaves: i32) -> f32
{
    var freq = fre;
    let  G = exp2(-.85);
    var amp = 1.;
    var noise = 0.;
    for (var i = 0; i < octaves; i++)
    {
        noise += amp * gradientNoise(p * freq, freq);
        freq *= 2.;
        amp *= G;
    }
    
    return noise;
}

// Tileable Worley fbm inspired by Andrew Schneider's Real-Time Volumetric Cloudscapes
// chapter in GPU Pro 7.
fn worleyFbm(p: vec3<f32>, freq: f32) -> f32
{
    return worleyNoise(p*freq, freq) * .625 +
        	 worleyNoise(p*freq*2., freq*2.) * .25 +
        	 worleyNoise(p*freq*4., freq*4.) * .125;
}

fn noise_helper( p: vec3<f32>) -> vec4<f32> {
    var col = vec4(0.);
    
    let freq = 4.;
    
    var pfbm = mix(1., perlinfbm(p, 4., 7), .5);
    pfbm = abs(pfbm * 2. - 1.); // billowy perlin noise
    
    col.g += worleyFbm(p, freq);
    col.b += worleyFbm(p, freq*2.);
    col.a += worleyFbm(p, freq*4.);
    col.r += remap(pfbm, 0., 1., col.g, 1.); // perlin-worley
    
    return vec4(col);
}


fn numericalMieFit(costh: f32) -> f32
{
    // This function was optimized to minimize (delta*delta)/reference in order to capture
    // the low intensity behavior.
    let bestParams = array<f32, 10>( 9.805233e-06, -6.500000e+01, -5.500000e+01, 8.194068e-01, 1.388198e-01, -8.370334e+01, 7.810083e+00, 2.054747e-03, 2.600563e-02, -4.552125e-12, );
    let p1 = costh + bestParams[3];
    let expValues = exp(vec4(bestParams[1] *costh+bestParams[2], bestParams[5] *p1*p1, bestParams[6] *costh, bestParams[9] *costh));
    let expValWeight= vec4(bestParams[0], bestParams[4], bestParams[7], bestParams[8]);
    return dot(expValues, expValWeight) * 0.25;
}

fn boxIntersection(ro: vec3<f32>, rd: vec3<f32>, boxSize: vec3<f32>) -> vec2<f32> {
    let m = 1.0 / rd; // can precompute if traversing a set of aligned boxes
    let n = m * ro;   // can precompute if traversing a set of aligned boxes
    let k = abs(m) * boxSize;
    let t1 = -n - k;
    let t2 = -n + k;
    let tN = max(max(t1.x, t1.y), t1.z);
    let tF = min(min(t2.x, t2.y), t2.z);
    if tN > tF || tF < 0.0 {
        return vec2(-1.0);
    }; // no intersection
    return vec2(tN, tF);
}

fn boxOut(ro: vec3<f32>, rd: vec3<f32>, boxSize: vec3<f32>) -> f32{
    let i = boxIntersection(ro, rd, boxSize);
    return max(0., max(i.x, i.y)) - min(max(0., i.y), max(0., i.x));
}

// fn hash(p: vec2<f32>) -> f32 {
//     var p3 = fract(vec3(p.xyx) * .1031);
//     p3 += dot(p3, p3.yzx + 33.33);
//     return fract((p3.x + p3.y) * p3.z);
// }

fn dsmix(a: f32, b:f32, t: f32) -> f32{
    return 6.*(a-b)*t*(t - 1.);
}
    
 fn noise(p: vec2<f32>) -> vec3<f32>{
    let cell = floor(p);
    let f = fract(p);
    let tr = cell + vec2(1., 1.);
    let tl = cell + vec2(0., 1.);
    let br = cell + vec2(1., 0.);
    let bl = cell + vec2(0., 0.);
    let htr = hash(tr);
    let htl = hash(tl);
    let hbr = hash(br);
    let hbl = hash(bl);
    let bx = smoothstep(0., 1., f.x);
    let by = smoothstep(0., 1., f.y);
    let mt = mix(htl, htr, bx);
    let mb = mix(hbl, hbr, bx);
    let dxt = dsmix(htl,htr,p.x);
    let dxb = dsmix(hbl,hbr,p.x);
    let dx = mix(dxb,dxt,smoothstep(0.,1.,f.y));
    let dy = dsmix(mb,mt,p.y);
    return vec3(mix(mb, mt, by),dx,dy);
}

// fn clip(a: f32, t: f32) -> f32 {
//     return (a-t) / (1.-t);
// }
fn clip(a: vec3<f32>, t: f32) -> vec3<f32> {
    return (a-t) / (1.-t);
}

fn fbm(p: vec2<f32>) -> vec3<f32>{
    var l: vec3<f32> = clip(0.5*(noise(p)+noise(p*2.)), 0.65);
    l += noise(p*4.)*0.5;    
    l.x += noise(p*5.).x*0.3;    
    l.x += noise(p*7.).x*0.2;
    l.x += noise(p*20.).x*0.4;
    l.x += noise(p*30.).x*0.2;
    let n = normalize(l.yz);
    l.y = n.x;
    l.z = n.y;

 return l;
}
    

fn n(p: vec3<f32>) -> f32{
    let n = noise_helper(p);

    let perlinWorley = n.x;

    // worley fbms with different frequencies
    let worley = n.yzw;
    let wfbm = worley.x * .625 +
    worley.y * .125 +
    worley.z * .25; 

    // cloud shape modeled after the GPU Pro 7 chapter
    var cloud = remap(perlinWorley, wfbm - 1., 1., 0., 1.);
    cloud = remap(cloud, .85, 1., 0., 1.); // fake cloud coverage
    return cloud;
}
    
@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {




    // var T = fbm( world_position.xy * 4.)*0.5;

    
    return vec4(textureSample(base_color_texture, base_color_sampler, uv));
}
