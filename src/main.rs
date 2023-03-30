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
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(CloudPlugin)
        .add_startup_system(setup)
        .add_plugin(FlyCameraPlugin)
        .add_plugin(MaterialPlugin::<CubemapMaterial>::default())
        .add_plugin(SkyBoxPlugin {})
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .run();
}

#[derive(Component, Default)]
struct CameraController {}

#[derive(Component, Default)]
struct Cloud {
    handle: Handle<CloudMaterial>,
}

struct CloudPlugin;

impl Plugin for CloudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<CloudMaterial>::default());
        app.add_system(update_cloud);
    }
}

fn update_cloud(
    cam: Query<&Transform, With<CameraController>>,
    clouds: Query<(&Cloud, &Transform)>,
    mut materials: ResMut<Assets<CloudMaterial>>,
    time: Res<Time>,
) {
    let camera_position = cam.get_single().unwrap().translation;
    for (cloud, transform) in &clouds {
        if let Some(material) = materials.get_mut(&cloud.handle) {
            material.camera_position = camera_position;
            material.time = time.raw_elapsed_seconds();
            material.aabb_position = transform.translation;
            material.scale = transform.scale;
            let relative = (camera_position - transform.translation).abs();
            material.cull_mode = if relative.x > transform.scale.x
                || relative.y > transform.scale.y
                || relative.z > transform.scale.z
            {
                Some(bevy::render::render_resource::Face::Front)
            } else {
                Some(bevy::render::render_resource::Face::Back)
            };
        }
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
        let res = [250; 3];
        let sdf_data = cloud_gen::new(res)
            .iter()
            .flatten()
            .flatten()
            .map(|v| [v.x.to_ne_bytes(), v.y.to_ne_bytes()])
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
            TextureFormat::Rg32Float,
        ));
        let material = cloud_materials.add(CloudMaterial {
            sdf: Some(texture.clone()),
            texture_dimensions: vec3(res[0] as f32, res[1] as f32, res[2] as f32),
            sun_direction: vec3(1., 1., 0.).normalize(),
            alpha_mode: AlphaMode::Blend,
            ..default()
        });

        commands.spawn((
            Cloud {
                handle: material.clone(),
            },
            MaterialMeshBundle {
                // mesh: meshes.add(cloud_gen::new(100.)),
                mesh: meshes.add(shape::Box::new(1., 1., 1.).into()),
                material,
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            },
        ));
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
        BloomSettings {
            intensity: 0.5,
            ..default()
        },
    ));
    camera.insert(CameraController::default());
    camera.insert(FlyCamera {
        sensitivity: 10.,
        ..default()
    });
}
