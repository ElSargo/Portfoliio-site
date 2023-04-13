use crate::sdf::sdf as cloud_sdf;
use crate::{noise, CameraController};
use bevy::{
    math::{vec3, vec4},
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat},
};

#[derive(Component, Default)]
struct RMCloud {
    handle: Handle<RMCloudMaterial>,
}

pub struct RMCloudPlugin;
impl Plugin for RMCloudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<RMCloudMaterial>::default());
        app.add_system(
            |cam: Query<&Transform, With<CameraController>>,
             clouds: Query<(&RMCloud, &Transform)>,
             mut cloud_materials: ResMut<Assets<RMCloudMaterial>>,
             time: Res<Time>| {
                let camera_position = cam.get_single().unwrap().translation;
                for (cloud, transform) in &clouds {
                    if let Some(material) = cloud_materials.get_mut(&cloud.handle) {
                        material.camera_position = camera_position;
                        material.time = time.raw_elapsed_seconds();
                        material.aabb_position = transform.translation;
                        material.scale = transform.scale;
                    }
                }
            },
        );

        app.add_startup_system(
            |mut commands: Commands,
             // asset_server: Res<AssetServer>,
             mut meshes: ResMut<Assets<Mesh>>,
             // mut materials: ResMut<Assets<StandardMaterial>>,
             mut cloud_materials: ResMut<Assets<RMCloudMaterial>>,
             // mut noise_materials: ResMut<Assets<NoiseMaterial>>,
             mut images: ResMut<Assets<Image>>| {
                {
                    let res = [30; 3];

                    let sdf_data = new_cloud_data(res)
                        .iter()
                        .flatten()
                        .flatten()
                        .map(|v| {
                            [
                                v.x.to_ne_bytes(),
                                v.y.to_ne_bytes(),
                                v.z.to_ne_bytes(),
                                v.w.to_ne_bytes(),
                            ]
                        })
                        .flatten()
                        .flatten()
                        .collect::<Vec<u8>>();
                    let texture = images.add(Image::new(
                        Extent3d {
                            width: res[0] as u32,
                            height: res[1] as u32,
                            depth_or_array_layers: res[2] as u32,
                        },
                        TextureDimension::D3,
                        sdf_data,
                        TextureFormat::Rgba32Float,
                    ));
                    let material = cloud_materials.add(RMCloudMaterial {
                        sdf: Some(texture.clone()),
                        texture_dimensions: vec3(res[0] as f32, res[1] as f32, res[2] as f32),
                        sun_direction: vec3(1., 1., 0.).normalize(),
                        ..default()
                    });

                    commands.spawn((
                        RMCloud {
                            handle: material.clone(),
                        },
                        MaterialMeshBundle {
                            // mesh: meshes.add(cloud_gen::new(100.)),
                            mesh: meshes.add(shape::Box::new(1., 1., 1.).into()),
                            material,
                            transform: {
                                let mut t = Transform::from_xyz(100.0, 100.0, 200.0);
                                t.scale = vec3(100., 100., 100.);
                                t
                            },

                            ..default()
                        },
                    ));
                };
            },
        );
    }
}

// A 16^3 chunk with 1-voxel boundary padding.

// This chunk will cover just a single octant of a sphere SDF (radius 15).
pub fn new_cloud_data(buffer_dimensions: [usize; 3]) -> Vec<Vec<Vec<Vec4>>> {
    let resolution = vec3(
        buffer_dimensions[0] as f32,
        buffer_dimensions[1] as f32,
        buffer_dimensions[2] as f32,
    );

    let mut data = vec![
        vec![vec![vec4(1., 1., 1., 1.); buffer_dimensions[2]]; buffer_dimensions[1]];
        buffer_dimensions[0]
    ];
    for x in 0..buffer_dimensions[0] {
        for y in 0..buffer_dimensions[1] {
            for z in 0..buffer_dimensions[2] {
                let p = coord_to_pos([x, y, z], resolution);
                let d = cloud_sdf(p);
                let n = noise::fbmd(p, Vec3::ONE * 1000.).x;
                data[x][y][z] = vec4(d, 0., n, 0.);
            }
        }
    }
    // Sun light info requires sdf and density info

    let sun_base = vec3(-1., 0.3, 1.).normalize();

    let mut sun_directions = Vec::with_capacity(27);
    // sun_directions.push((sun_base, mie(1.)));
    let theta = 0.15;
    let angles = [-theta, 0.0, theta];
    for x in angles {
        for y in angles {
            for z in angles {
                let direction = rotate(sun_base, x, y, z);
                sun_directions.push((direction, mie(sun_base.dot(direction))));
            }
        }
    }

    // Sun raymarching
    for x in 0..buffer_dimensions[0] {
        for y in 0..buffer_dimensions[1] {
            for z in 0..buffer_dimensions[2] {
                let mut total = 0.;
                let dt = 2. / resolution.x.min(resolution.y).min(resolution.z);
                for (sun_direction, phase) in sun_directions.iter() {
                    let mut t = 1.;
                    let mut p = coord_to_pos([x, y, z], resolution);
                    let mut sample_point = [x, y, z];
                    while let Some(samp) = data
                        .get(sample_point[0])
                        .and_then(|slice| slice.get(sample_point[1]))
                        .and_then(|row| row.get(sample_point[2]))
                    {
                        if p.x.abs() > 1. || p.y.abs() > 1. || p.z.abs() > 1. {
                            break;
                        }

                        let distance = samp.x;
                        let noise = samp.z;
                        if distance < 0.0 {
                            t *= (noise * dt).max(0.)
                        }
                        p += *sun_direction * dt * *phase;
                        sample_point = pos_to_coord(p, resolution);
                    }
                    total += t;
                }
                data[x][y][z].y = total / sun_directions.len() as f32;
            }
        }
    }
    data
}

#[allow(dead_code)]
fn rotate(v: Vec3, x: f32, y: f32, z: f32) -> Vec3 {
    Mat3::from_euler(bevy::prelude::EulerRot::XYZ, x, y, z) * v
}

fn mie(costh: f32) -> f32 {
    // This function was optimized to minimize (delta*delta)/reference in order to capture
    // the low intensity behavior.
    let params = [
        9.805233e-06,
        -6.500000e+01,
        -5.500000e+01,
        8.194068e-01,
        1.388198e-01,
        -8.370334e+01,
        7.810083e+00,
        2.054747e-03,
        2.600563e-02,
        -4.552125e-12,
    ];

    let p1 = costh + params[3];
    let exp_values = vec4(
        params[1] * costh + params[2],
        params[5] * p1 * p1,
        params[6] * costh,
        params[9] * costh,
    )
    .exp();
    let exp_val_weight = vec4(params[0], params[4], params[7], params[8]);
    return exp_values.dot(exp_val_weight) * 0.25;
}

pub fn coord_to_pos<T: AsF32 + Copy>(coord: [T; 3], res: Vec3) -> Vec3 {
    (vec3(coord[0].as_f32(), coord[1].as_f32(), coord[2].as_f32()) / res - 0.5) * 2.
}

pub fn pos_to_coord(p: Vec3, res: Vec3) -> [usize; 3] {
    let c = ((p / 2.) + 0.5) * res;
    [
        c.x.round() as usize,
        c.y.round() as usize,
        c.z.round() as usize,
    ]
}

pub trait AsF32 {
    fn as_f32(self) -> f32;
}
impl AsF32 for f32 {
    fn as_f32(self) -> f32 {
        self
    }
}
impl AsF32 for u32 {
    fn as_f32(self) -> f32 {
        self as f32
    }
}
impl AsF32 for usize {
    fn as_f32(self) -> f32 {
        self as f32
    }
}

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for RMCloudMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/cloud.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone, Default, Reflect)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct RMCloudMaterial {
    #[uniform(0)]
    pub sun_direction: Vec3,
    #[uniform(0)]
    pub camera_position: Vec3,
    #[uniform(0)]
    pub aabb_position: Vec3,
    #[uniform(0)]
    pub texture_dimensions: Vec3,
    #[uniform(0)]
    pub scale: Vec3,
    #[uniform(0)]
    pub time: f32,
    #[texture(1, dimension = "3d")]
    #[sampler(2)]
    pub sdf: Option<Handle<Image>>,
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone, Default, Reflect)]
#[uuid = "f692fd8e-d598-45ab-8225-97e2a3f056e0"]
pub struct GodRayMaterial {
    #[uniform(0)]
    pub sun_direction: Vec3,
    #[uniform(0)]
    pub camera_position: Vec3,
    #[uniform(0)]
    pub scale: Vec3,
    #[uniform(0)]
    pub time: f32,
}
