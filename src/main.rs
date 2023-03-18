// use crate::skybox::CubemapMaterial;
// use skybox::SkyBoxPlugin;
use bevy::prelude::*;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use cloud_shader::RealTimeCloudMaterial;
use noise_shader::NoiseMaterial;
mod camera;
mod cloud_shader;
mod noise_shader;
mod skybox;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes: true,
            ..Default::default()
        }))
        .add_plugin(MaterialPlugin::<RealTimeCloudMaterial>::default())
        .add_plugin(MaterialPlugin::<NoiseMaterial>::default())
        // .add_plugin(WorldInspectorPlugin {})
        .add_startup_system(setup)
        .add_system(update_cloud)
        .add_plugin(FlyCameraPlugin)
        // .add_plugin(MaterialPlugin::<CubemapMaterial>::default())
        // .add_plugin(SkyBoxPlugin {})
        .run();
}

#[derive(Component, Default)]
struct CameraController {}

fn update_cloud(
    cam: Query<&Transform, With<CameraController>>,
    clouds: Query<(&Transform, &Cloud)>,
    mut materials: ResMut<Assets<RealTimeCloudMaterial>>,
) {
    let cam_pos = cam.get_single().unwrap().translation;
    for (transform, handle) in clouds.iter() {
        let material = match materials.get_mut(&handle.material) {
            Some(handle) => handle,
            None => {
                println!("No handle");
                continue;
            }
        };
        material.cam_pos = cam_pos;
        material.box_pos = transform.translation;
        material.box_size = transform.scale;
    }
}

#[derive(Component)]
struct Cloud {
    material: Handle<RealTimeCloudMaterial>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cloud_materials: ResMut<Assets<RealTimeCloudMaterial>>,
    mut noise_materials: ResMut<Assets<NoiseMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let sun_dir = Vec3::new(1., -1., 0.5);

    // cube
    let mat = cloud_materials.add(RealTimeCloudMaterial {
        color: Color::BLUE,
        sun_dir: sun_dir.normalize(),
        cam_pos: Vec3::new(-2.0, 2.5, 5.0),
        box_pos: Vec3::new(0., 0., 0.),
        box_size: Vec3::new(0., 0., 0.),
        color_texture: Some(asset_server.load("textures/cloud.png")),
        alpha_mode: AlphaMode::Blend,
    });

    commands
        .spawn(MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: mat.clone(),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        })
        .insert(Cloud { material: mat });

    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: noise_materials.add(NoiseMaterial {
            color_texture: Some(asset_server.load("textures/cloud.png")),
            alpha_mode: AlphaMode::Opaque,
            octaves: 1,
            scale: 1.,
            contribution: 1.,
            falloff: 1.,
            threshold: 1.,
        }),
        transform: Transform::from_xyz(1.5, 0.5, 0.0),
        ..default()
    });

    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Plane {
            subdivisions: 1,
            size: 2000.0,
        })),
        material: noise_materials.add(NoiseMaterial {
            color_texture: Some(asset_server.load("textures/cloud.png")),
            alpha_mode: AlphaMode::Opaque,
            octaves: 1,
            scale: 1.,
            contribution: 1.,
            falloff: 1.,
            threshold: 1.,
        }),
        transform: Transform::from_xyz(1.5, 0.5, 0.0),
        ..default()
    });

    // ambient light
    // NOTE: The ambient light is used to scale how bright the environment map is so with a bright
    // environment map, use an appropriate colour and brightness to match
    commands.insert_resource(AmbientLight {
        color: Color::rgb_u8(210, 220, 240),
        brightness: 1.0,
    });

    // light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(1., 0.8, 0.6),
            illuminance: 20_000.0,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_at(sun_dir, Vec3::Y),
        ..default()
    });
    // camera
    let mut camera = commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        camera: Camera {
            hdr: true,
            ..default()
        },
        ..default()
    });
    camera.insert(CameraController::default());
    camera.insert(FlyCamera::default());
}
