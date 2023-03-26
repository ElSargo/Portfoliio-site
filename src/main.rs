// use crate::skybox::CubemapMaterial;
// use skybox::SkyBoxPlugin;
use bevy::{
    core_pipeline::bloom::BloomSettings,
    // diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::vec3,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use cloud_shader::CloudMaterial;
use noise_shader::NoiseMaterial;
use skybox::{CubemapMaterial, SkyBoxPlugin};
mod camera;
mod cloud_gen;
mod cloud_shader;
mod noise;
mod noise_shader;
mod sdf;
mod skybox;
mod test_cloud_shader;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes: true,
            ..Default::default()
        }))
        .add_plugin(MaterialPlugin::<CloudMaterial>::default())
        .add_plugin(MaterialPlugin::<NoiseMaterial>::default())
        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(setup)
        .add_system(update_cloud)
        .add_plugin(FlyCameraPlugin)
        .add_plugin(MaterialPlugin::<CubemapMaterial>::default())
        .add_plugin(SkyBoxPlugin {})
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .run();
}

#[derive(Component, Default)]
struct CameraController {}

fn update_cloud(
    cam: Query<&Transform, With<CameraController>>,
    mut materials: ResMut<Assets<CloudMaterial>>,
    time: Res<Time>,
) {
    let camera_position = cam.get_single().unwrap().translation;
    for material in materials.iter_mut() {
        material.1.camera_position = camera_position;
        material.1.time = time.raw_elapsed_seconds();
        // println!("{:?}", time.raw_elapsed_seconds());
    }
}

fn setup(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
    mut cloud_materials: ResMut<Assets<CloudMaterial>>,
    // mut noise_materials: ResMut<Assets<NoiseMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let sun_dir = Vec3::new(-1., -0.2, 0.1);

    // cube
    // commands.spawn(MaterialMeshBundle {
    //     mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    //     material: noise_materials.add(NoiseMaterial {
    //         color_texture: Some(asset_server.load("textures/cloud.png")),
    //         alpha_mode: AlphaMode::Opaque,
    //         octaves: 1,
    //         scale: 1.,
    //         contribution: 1.,
    //         falloff: 1.,
    //         threshold: 1.,
    //     }),
    //     transform: Transform::from_xyz(1.5, 0.5, 0.0),
    //     ..default()
    // });

    {
        let res = 127;
        let resf = res as f32;
        commands.spawn(MaterialMeshBundle {
            // mesh: meshes.add(cloud_gen::new(100.)),
            mesh: meshes.add(shape::Box::new(1., 1., 1.).into()),
            material: cloud_materials.add(CloudMaterial {
                sdf: Some(
                    images.add(Image::new(
                        Extent3d {
                            width: res * res,
                            height: res,
                            depth_or_array_layers: 1,
                        },
                        TextureDimension::D2,
                        cloud_gen::new(resf)
                            .iter()
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
                            .collect::<Vec<u8>>(),
                        TextureFormat::Rgba32Float,
                    )),
                ),
                texture_dimensions: vec3(resf, resf, resf),
                alpha_mode: AlphaMode::Blend,
                color: Color::rgba(1., 1., 1., 1.),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        })
    };

    // ambient light
    // NOTE: The ambient light is used to scale how bright the environment map is so with a bright
    // environment map, use an appropriate colour and brightness to match
    commands.insert_resource(AmbientLight {
        color: Color::rgb(0.54, 0.8, 1.),
        brightness: 1.0,
    });

    // light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(2.2, 2.05, 1.9),
            illuminance: 20_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_at(sun_dir, Vec3::Y),
        ..default()
    });
    // camera
    let mut camera = commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            camera: Camera {
                hdr: true,
                ..default()
            },
            // tonemapping: bevy::core_pipeline::tonemapping::Tonemapping::AcesFitted,
            ..default()
        },
        BloomSettings { ..default() },
    ));
    camera.insert(CameraController::default());
    camera.insert(FlyCamera {
        sensitivity: 10.,
        ..default()
    });
}
