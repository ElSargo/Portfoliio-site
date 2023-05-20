use bevy::{
    math::{vec2, vec3},
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef, TextureFormat},
};
use rand::prelude::*;
use std::ops::{Add, Mul, Sub};

use crate::{noise, CameraController};

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
        const TEXTURE_RES: usize = 200;
        app.add_startup_system(
            |mut materials: ResMut<Assets<CloudBlobMaterial>>,
             mut commands: Commands,
             mut meshes: ResMut<Assets<Mesh>>,
             mut images: ResMut<Assets<Image>>| {
                let path = "assets/noise_data";
                let data = match std::fs::read(path) {
                    Ok(data) => data,
                    Err(_) => {
                        let mut data = Box::new([[[0.0; TEXTURE_RES]; TEXTURE_RES]; TEXTURE_RES]);
                        for x in 0..TEXTURE_RES {
                            for y in 0..TEXTURE_RES {
                                for z in 0..TEXTURE_RES {
                                    let sample_pos = vec3(
                                        x as f32 / TEXTURE_RES as f32,
                                        y as f32 / TEXTURE_RES as f32,
                                        z as f32 / TEXTURE_RES as f32,
                                    ) * 10.;
                                    data[x][y][z] = mix(
                                        noise::fbmd(sample_pos).x,
                                        noise::wfbm(sample_pos * 0.5, Vec3::ONE * 100.),
                                        0.7,
                                    )
                                }
                            }
                        }
                        let data = data
                            .iter()
                            .flatten()
                            .flatten()
                            .flat_map(|f| f.to_ne_bytes())
                            .collect();
                        if let Err(e) = std::fs::write(path, &data) {
                            println!("Error writing noise data {:?}", e);
                        }
                        data
                    }
                };

                let texture = images.add(Image::new(
                    bevy::render::render_resource::Extent3d {
                        width: TEXTURE_RES as u32,
                        height: TEXTURE_RES as u32,
                        depth_or_array_layers: TEXTURE_RES as u32,
                    },
                    bevy::render::render_resource::TextureDimension::D3,
                    data,
                    TextureFormat::R32Float,
                ));
                let mesh = meshes.add(
                    shape::UVSphere {
                        radius: 1.0,
                        sectors: SECTORS,
                        stacks: STACKS,
                    }
                    .into(),
                );
                let mut rng = thread_rng();
                for _ in 0..200 {
                    let xz =
                        vec2(rng.gen(), rng.gen()).add(vec2(-0.5, -0.5)) * vec2(10000., 10000.);
                    let y = (10_000. - xz.length()).sqrt().sub(10.).mul(10.);
                    let pos = vec3(xz.x, y, xz.y);

                    let material = materials.add(CloudBlobMaterial {
                        noise: Some(texture.clone()),
                        ..default()
                    });
                    commands.spawn((
                        CloudBlob {
                            handle: material.clone(),
                        },
                        MaterialMeshBundle {
                            mesh: mesh.clone(),
                            material,
                            transform: Transform::from_xyz(pos.x, pos.y, pos.z)
                                .with_scale(vec3(400., 300., 400.) * rng.gen_range(0.5..1.0)),
                            ..default()
                        },
                    ));
                }
                for _ in 0..20 {
                    let xz =
                        vec2(rng.gen(), rng.gen()).add(vec2(-0.5, -0.5)) * vec2(10000., 10000.);
                    let y = (10_000. - xz.length()).sqrt().sub(10.).mul(10.);
                    let pos = vec3(xz.x, y, xz.y);
                    for (scale, offset) in [
                        (
                            vec3(1000., 400., 1000.) * rng.gen_range(0.5..1.0),
                            vec3(0., 0., 0.),
                        ),
                        (
                            vec3(400., 400., 400.) * rng.gen_range(0.75..1.0),
                            vec3(0., 200., 0.),
                        ),
                        (
                            vec3(200., 200., 200.) * rng.gen_range(0.5..1.0),
                            vec3(200., 50., 200.) * rng.gen_range(0.75..1.0),
                        ),
                        (
                            vec3(200., 200., 200.) * rng.gen_range(0.5..1.0),
                            vec3(200., 50., -200.) * rng.gen_range(0.75..1.0),
                        ),
                        (
                            vec3(200., 200., 200.) * rng.gen_range(0.5..1.0),
                            vec3(-200., 50., 200.) * rng.gen_range(0.75..1.0),
                        ),
                        (
                            vec3(200., 200., 200.) * rng.gen_range(0.5..1.0),
                            vec3(-200., 50., -200.) * rng.gen_range(0.75..1.0),
                        ),
                    ] {
                        // let offset = (vec3(rng.gen(), rng.gen(), rng.gen()) - 0.5) * 400.;
                        let material = materials.add(CloudBlobMaterial {
                            noise: Some(texture.clone()),
                            ..default()
                        });
                        commands.spawn((
                            CloudBlob {
                                handle: material.clone(),
                            },
                            MaterialMeshBundle {
                                mesh: mesh.clone(),
                                material,
                                transform: Transform::from_xyz(
                                    pos.x + offset.x,
                                    pos.y + offset.y,
                                    pos.z + offset.z,
                                )
                                .with_scale(scale),
                                ..default()
                            },
                        ));
                    }
                }
            },
        );
    }
}

fn mix(a: f32, b: f32, t: f32) -> f32 {
    a * (1. - t) + b * t
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
        AlphaMode::Premultiplied
    }
}
