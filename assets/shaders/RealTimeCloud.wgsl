struct RealTimeCloudMaterial {
    color: vec4<f32>,
    sun_dir: vec3<f32>,
    cam_pos: vec3<f32>,
    box_pos: vec3<f32>,
    box_size: vec3<f32>,
};

@group(1) @binding(0)
var<uniform> material: RealTimeCloudMaterial;
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


    
@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let sun = normalize(vec3(1.,1.,1.));
    let pos = world_position.xyz;
    let nor = world_normal;
    let ro = material.cam_pos;
    let rd = normalize(pos - ro);


    let bounds = boxIntersection(ro-material.box_pos, rd, material.box_size*0.5);
    
    var col = vec3(.1);
    var T = max( 0. , bounds.x );
    var alpha = 0.;
    T += abs(hash(uv*23.123123))*0.02*T;
    let dt = 0.1;
    while T < bounds.y {
        // col += 0.1*dt*T;
        let p = ro+rd*T;

        var d = 2.*(abs(p.y-material.box_pos.y)-material.box_size.y);
        d += n(p*vec3(0.5,1.,0.5));


        col += 0.1*(.5 - max(0., n(p*vec3(0.5,1.,0.5)+0.1)));
        
        alpha += 0.2*max(0., 3. * d);
        if alpha > 0.9 {
            break;
        }
        T += dt*T;
    }

    
    
    return vec4(1.);
    // return vec4(1.,1.,1.,1.);
}
