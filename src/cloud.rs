use bevy::math::vec2;

// use crate::noise::fbmd;
use crate::{noise, CameraController};
use bevy::{
    math::{vec3, vec4},
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat},
};

#[derive(Component, Default, Reflect)]
struct RMCloud {
    pub handle: Handle<RMCloudMaterial>,
    pub shadow_dist: f32,
    pub shadow_coef: f32,
    pub sun_pen: f32,
    pub worley_factor: f32,
    pub value_factor: f32,
    pub cloud_coef: f32,
    pub cloud_height: f32,
}

pub struct RMCloudPlugin;
impl Plugin for RMCloudPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RMCloud>();
        app.add_plugin(MaterialPlugin::<RMCloudMaterial>::default());
        app.add_system(
            |cam: Query<&Transform, With<CameraController>>,
             clouds: Query<(&RMCloud, &Transform)>,
             sun: Query<&Transform, With<DirectionalLight>>,
             mut cloud_materials: ResMut<Assets<RMCloudMaterial>>,
             time: Res<Time>| {
                let camera_position = cam.get_single().unwrap().translation;
                let sun_dir = sun.get_single().unwrap().forward();
                for (cloud, _transform) in &clouds {
                    if let Some(material) = cloud_materials.get_mut(&cloud.handle) {
                        material.camera_position = camera_position;
                        material.time = time.raw_elapsed_seconds();
                        material.sun_direction = sun_dir;
                    }
                }
            },
        );

        app.add_system(
            |mut clouds: Query<&mut RMCloud>, mut materials: ResMut<Assets<RMCloudMaterial>>| {
                for cloud in clouds.iter_mut() {
                    match materials.get_mut(&cloud.handle) {
                        Some(material) => {
                            material.shadow_dist = cloud.shadow_dist;
                            material.shadow_coef = cloud.shadow_coef;
                            material.worley_factor = cloud.worley_factor;
                            material.value_factor = cloud.value_factor;
                            material.cloud_coef = cloud.cloud_coef;
                            material.cloud_height = cloud.cloud_height;
                            material.sun_pen = cloud.sun_pen;
                        }
                        None => {}
                    };
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
                let res = (1000, 1000);
                let re3 = 2;

                let w3d = {
                    images.add(Image::new(
                        Extent3d {
                            width: re3 as u32,
                            height: re3 as u32,
                            depth_or_array_layers: re3 as u32,
                        },
                        TextureDimension::D3,
                        w3noise(re3)
                            .iter()
                            .flat_map(|f| f.to_ne_bytes())
                            .collect::<Vec<u8>>(),
                        TextureFormat::R32Float,
                    ))
                };

                let mut make_image = |data: &[f32]| {
                    images.add(Image::new(
                        Extent3d {
                            width: res.0 as u32,
                            height: res.1 as u32,
                            depth_or_array_layers: 1,
                        },
                        TextureDimension::D2,
                        data.iter()
                            .flat_map(|f| f.to_ne_bytes())
                            .collect::<Vec<u8>>(),
                        TextureFormat::R32Float,
                    ))
                };

                let wnoise = worley_texture_data(res, vec2(5., 5.));
                let vnoise = value_texture_data(res, vec2(5., 5.));
                let worley = make_image(&wnoise);
                let value = make_image(&vnoise);
                let material = cloud_materials.add(RMCloudMaterial {
                    worley: Some(worley.clone()),
                    value: Some(value.clone()),
                    w3d: Some(w3d),
                    sun_direction: vec3(1., 1., 0.).normalize(),
                    ..default()
                });

                commands.spawn((
                    RMCloud {
                        handle: material.clone(),
                        shadow_dist: 50.0,
                        shadow_coef: 0.1,
                        sun_pen: 30.,
                        worley_factor: 0.1,
                        value_factor: 0.0,
                        cloud_coef: 0.2,
                        cloud_height: 0.2,
                    },
                    MaterialMeshBundle {
                        // mesh: meshes.add(cloud_gen::new(100.)),
                        mesh: meshes.add(
                            shape::Plane {
                                size: 1000.0,
                                ..default()
                            }
                            .into(),
                        ),
                        material,

                        ..default()
                    },
                ));
            },
        );
    }
}

fn w3noise(res: usize) -> Vec<f32> {
    let scale = vec3(10., 10., 10.);
    let resolution = vec3(res as f32, res as f32, res as f32);
    (0..res)
        .flat_map(|x| (0..res).flat_map(move |y| (0..res).map(move |z| (x, y, z))))
        .map(|(x, y, z)| {
            let p = vec3(x as f32, y as f32, z as f32) / resolution * scale;
            noise::wfbm(p, scale)
        })
        .collect()
}

// A 16^3 chunk with 1-voxel boundary padding.

// This chunk will cover just a single octant of a sphere SDF (radius 15).

pub fn worley_texture_data(buffer_dimensions: (usize, usize), scale: Vec2) -> Vec<f32> {
    let resolution = vec2(buffer_dimensions.0 as f32, buffer_dimensions.1 as f32);
    (0..buffer_dimensions.0)
        .flat_map(move |x| (0..buffer_dimensions.1).map(move |y| (x, y)))
        .map(|(x, y)| {
            let p = vec2(x.as_f32(), y.as_f32()) / resolution * scale;
            noise::wfbm(
                p.extend(0.0),
                vec3(resolution.x, resolution.y, resolution.y),
            )
        })
        .collect()
}

pub fn value_texture_data(buffer_dimensions: (usize, usize), scale: Vec2) -> Vec<f32> {
    let resolution = vec2(buffer_dimensions.0 as f32, buffer_dimensions.1 as f32);
    println!("{resolution}");
    (0..buffer_dimensions.0)
        .flat_map(|x| (0..buffer_dimensions.1).map(move |y| (x, y)))
        .map(|(x, y)| {
            let p = vec2(x.as_f32(), y.as_f32()) / resolution * scale;
            // let d = cloud_sdf(p);
            noise::value_fbm(
                p.extend(0.0),
                vec3(resolution.x, resolution.y, resolution.y),
            )
        })
        .collect()
}

// pub fn new_cloud_data(buffer_dimensions: [usize; 3]) -> Vec<Vec4> {
//     let resolution = vec3(
//         buffer_dimensions[0] as f32,
//         buffer_dimensions[1] as f32,
//         buffer_dimensions[2] as f32,
//     );

//     let data = Vec::from_iter((0..buffer_dimensions[0]).map(|_| {
//         (0..buffer_dimensions[1])
//             .map(|_| {
//                 (0..buffer_dimensions[2])
//                     .map(|_| Mutex::new(vec4(1., 1., 1., 1.)))
//                     .collect_vec()
//             })
//             .collect_vec()
//     }));
//     (0..buffer_dimensions[0])
//         .flat_map(move |x| {
//             (0..buffer_dimensions[1])
//                 .flat_map(move |y| (0..buffer_dimensions[2]).map(move |z| (x, y, z)))
//         })
//         .par_bridge()
//         .for_each(|(x, y, z)| {
//             let p = coord_to_pos([x, y, z], resolution);
//             // let d = cloud_sdf(p);
//             let sca = vec3(0.50, 0.50, 0.50) / 100.0 * resolution;
//             let n = ((noise::wfbm(p * sca, Vec3::ONE * 1000.0)
//                 * (2.0 + fbmd(p * sca + 110.1231231).x)
//                 * 0.5)/*
//              * ((1.0 - (-4.0 * (p.y + 1.0)).exp()) * ((-p.y).exp() - 0.37))*/)
//                 .clamp(0.0, 3.0);
//             *data[x][y][z].lock() = vec4(n, 0., n, 0.);
//         });

//     // for x in 0..buffer_dimensions[0] {
//     //     for y in 0..buffer_dimensions[1] {
//     //         for z in 0..buffer_dimensions[2] {
//     //         }
//     //     }
//     // }
//     // Sun light info requires sdf and density info

//     let sun_base = vec3(-0., 2., 0.).normalize();

//     let mut sun_directions = Vec::with_capacity(27);
//     sun_directions.push((sun_base, mie(1.)));
//     // let theta = 0.15;
//     // let angles = [-theta, 0.0, theta];
//     // for x in angles {
//     //     for y in angles {
//     //         for z in angles {
//     //             let direction = rotate(sun_base, x, y, z);
//     //             sun_directions.push((direction, mie(sun_base.dot(direction))));
//     //         }
//     //     }
//     // }

//     // Sun raymarching

//     (0..buffer_dimensions[0])
//         .flat_map(|x| {
//             (0..buffer_dimensions[1])
//                 .flat_map(move |y| (0..buffer_dimensions[2]).map(move |z| (x, y, z)))
//         })
//         .par_bridge()
//         .for_each(|(x, y, z)| {
//             let mut total = 0.;
//             let dt = 2. / resolution.x.min(resolution.y).min(resolution.z) * 0.1;
//             for (sun_direction, phase) in sun_directions.iter() {
//                 let mut t = 1.;
//                 let mut p = coord_to_pos([x, y, z], resolution);
//                 let mut sample_point = [x, y, z];
//                 while let Some(samp) = data
//                     .get(sample_point[0])
//                     .and_then(|slice| slice.get(sample_point[1]))
//                     .and_then(|row| row.get(sample_point[2]))
//                 {
//                     let samp = samp.lock();
//                     if p.x.abs() > 1. || p.y.abs() > 1. || p.z.abs() > 1. {
//                         break;
//                     }

//                     let dm = 0.5;
//                     let noise = 0.0_f32.max(samp.z - dm);
//                     // if noise > 0.0 {
//                     t += noise * dt * 5.0;
//                     // println!("{noise} {t}");
//                     // }
//                     p += *sun_direction * dt * *phase;
//                     sample_point = pos_to_coord(p, resolution);
//                 }
//                 total += t;
//             }
//             data[x][y][z].lock().y = total / sun_directions.len() as f32;
//         });
//     data.iter()
//         .flat_map(|row_col| {
//             row_col
//                 .iter()
//                 .flat_map(|row| row.iter().map(|c| c.lock().clone()))
//         })
//         .collect_vec()
// }

#[allow(dead_code)]
fn rotate(v: Vec3, x: f32, y: f32, z: f32) -> Vec3 {
    Mat3::from_euler(bevy::prelude::EulerRot::XYZ, x, y, z) * v
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
    pub time: f32,
    #[uniform(0)]
    pub shadow_dist: f32,
    #[uniform(0)]
    pub shadow_coef: f32,
    #[uniform(0)]
    pub sun_pen: f32,
    #[uniform(0)]
    pub worley_factor: f32,
    #[uniform(0)]
    pub value_factor: f32,
    #[uniform(0)]
    pub cloud_coef: f32,
    #[uniform(0)]
    pub cloud_height: f32,

    #[texture(1)]
    #[sampler(2)]
    pub worley: Option<Handle<Image>>,
    #[texture(3)]
    #[sampler(4)]
    pub value: Option<Handle<Image>>,
    #[texture(5, dimension = "3d")]
    #[sampler(6)]
    pub w3d: Option<Handle<Image>>,
}
