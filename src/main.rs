// orbital scene

use bevy::{
    core_pipeline::bloom::BloomSettings,
    math::vec3,
    prelude::*,
    render::render_resource::{AddressMode, FilterMode, SamplerDescriptor},
};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use cloud_blob::CloudBlobPlugin;
use skybox::{CubemapMaterial, SkyBoxPlugin};
use water::WaterPlugin;
mod camera;
mod cloud_blob;
// mod fin_cloud;
mod cloud;
mod noise;
mod noise_shader;
mod sdf;
mod skybox;
mod test_cloud_shader;
mod water;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes: true,
                    ..Default::default()
                })
                .set(ImagePlugin {
                    default_sampler: SamplerDescriptor {
                        address_mode_u: AddressMode::MirrorRepeat,
                        address_mode_v: AddressMode::MirrorRepeat,
                        address_mode_w: AddressMode::MirrorRepeat,
                        mag_filter: FilterMode::Linear,
                        min_filter: FilterMode::Linear,
                        mipmap_filter: FilterMode::Linear,

                        ..Default::default()
                    },
                }),
        )
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(cloud::RMCloudPlugin)
        // .add_plugin(fin_cloud::FinCloudPlugin)
        // .add_plugin(CloudBlobPlugin)
        // .add_plugin(WaterPlugin)
        .add_startup_system(setup)
        .add_plugin(FlyCameraPlugin)
        .add_plugin(MaterialPlugin::<CubemapMaterial>::default())
        .add_plugin(SkyBoxPlugin {})
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .run();
}

#[derive(Component, Default)]
pub struct CameraController {}
fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    // asset_server: Res<AssetServer>,
) {
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
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_at(vec3(-1., -0.3, 1.), Vec3::Y),
        ..default()
    });

    // test cube
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(shape::Cube { size: 2. }.into()),
        material: materials.add(Color::rgba(1., 0.6, 0.5, 1.).into()),
        transform: Transform::from_xyz(4., 0., 1.),
        ..default()
    });

    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0., 10., -10.).looking_at(Vec3::ZERO, Vec3::Y),
            projection: Projection::Perspective(PerspectiveProjection {
                far: 100_000.,
                ..default()
            }),
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: bevy::core_pipeline::tonemapping::Tonemapping::AcesFitted,
            ..default()
        },
        BloomSettings {
            intensity: 0.5,
            ..default()
        },
        CameraController::default(),
        FlyCamera { ..default() },
    ));
}
