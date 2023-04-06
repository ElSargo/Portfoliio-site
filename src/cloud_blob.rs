use bevy::{
    math::{vec2, vec3},
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef, TextureFormat},
};
use rand::prelude::*;
use std::ops::{Add, Mul, Sub};

use crate::{noise, rm_cloud::coord_to_pos, CameraController};

#[derive(Component, Default)]
struct CloudBlob {
    handle: Handle<CloudBlobMaterial>,
}

pub struct CloudBlobPlugin;

#[allow(dead_code)]
impl Plugin for CloudBlobPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<CloudBlobMaterial>::default());
        app.add_system(
            |camera: Query<&Transform, With<CameraController>>,
             sun: Query<&Transform, With<DirectionalLight>>,
             clouds: Query<(&CloudBlob, &Transform)>,
             mut materials: ResMut<Assets<CloudBlobMaterial>>,
             time: Res<Time>| {
                let camera_position = camera.get_single().unwrap().translation;
                let sun_facing = sun.get_single().unwrap().forward();
                for (cloud, transform) in &clouds {
                    if let Some(material) = materials.get_mut(&cloud.handle) {
                        material.camera_position = camera_position;
                        material.time = time.raw_elapsed_seconds();
                        material.scale = transform.scale;
                        material.sun_direction = sun_facing;
                    }
                }
            },
        );

        const SECTORS: usize = 10;
        const STACKS: usize = 10;
        const texture_res: usize = 50;
        app.add_startup_system(
            |mut materials: ResMut<Assets<CloudBlobMaterial>>,
             mut commands: Commands,
             mut meshes: ResMut<Assets<Mesh>>,
             mut images: ResMut<Assets<Image>>| {
                let mut noise_data = Box::new([[[0.0; texture_res]; texture_res]; texture_res]);
                for x in 0..texture_res {
                    for y in 0..texture_res {
                        for z in 0..texture_res {
                            noise_data[x][y][z] = noise::noise(
                                coord_to_pos(
                                    [x, y, z],
                                    Vec3 {
                                        x: texture_res as f32,
                                        y: texture_res as f32,
                                        z: texture_res as f32,
                                    },
                                ) / texture_res as f32
                                    * 6.,
                            )
                        }
                    }
                }
                let texture = images.add(Image::new(
                    bevy::render::render_resource::Extent3d {
                        width: texture_res as u32,
                        height: texture_res as u32,
                        depth_or_array_layers: texture_res as u32,
                    },
                    bevy::render::render_resource::TextureDimension::D3,
                    noise_data
                        .iter()
                        .flatten()
                        .flatten()
                        .map(|f| f.to_ne_bytes())
                        .flatten()
                        .collect(),
                    TextureFormat::R32Float,
                ));
                let material = materials.add(CloudBlobMaterial {
                    noise: Some(texture),
                    ..default()
                });
                let mesh = meshes.add(
                    shape::UVSphere {
                        radius: 1.0,
                        sectors: SECTORS,
                        stacks: STACKS,
                    }
                    .into(),
                );
                commands.spawn((
                    CloudBlob {
                        handle: material.clone(),
                    },
                    MaterialMeshBundle {
                        mesh: mesh.clone(),
                        transform: Transform::from_xyz(250., 1000., 400.)
                            .with_scale(vec3(400., 200., 400.)),
                        material: material.clone(),
                        ..default()
                    },
                ));
                let mut rng = thread_rng();
                for _ in 0..300 {
                    let xz =
                        vec2(rng.gen(), rng.gen()).add(vec2(-0.5, -0.5)) * vec2(10000., 10000.);
                    let y = (10_000. - xz.length()).sqrt().sub(10.).mul(10.);
                    let pos = vec3(xz.x, y, xz.y);
                    for _ in 0..1 {
                        let offset = (vec3(rng.gen(), rng.gen(), rng.gen()) - 0.5) * 400.;
                        commands.spawn((MaterialMeshBundle {
                            mesh: mesh.clone(),
                            material: material.clone(),
                            transform: Transform::from_xyz(
                                pos.x + offset.x,
                                pos.y + offset.y,
                                pos.z + offset.z,
                            )
                            .with_scale(vec3(800., 400., 800.) * rng.gen::<f32>()),
                            ..default()
                        },));
                    }
                }
            },
        );
    }
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone, Default, Reflect)]
#[uuid = "f690fd8e-d598-45ab-8225-97e2a3f056e0"]
pub struct CloudBlobMaterial {
    #[uniform(0)]
    pub sun_direction: Vec3,
    #[uniform(0)]
    pub camera_position: Vec3,
    #[uniform(0)]
    pub scale: Vec3,
    #[uniform(0)]
    pub time: f32,
    #[texture(1, dimension = "3d")]
    #[sampler(2)]
    pub noise: Option<Handle<Image>>,
}

impl Material for CloudBlobMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/cloud_blob.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
