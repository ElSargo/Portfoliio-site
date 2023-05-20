use std::f32::consts::PI;

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
};

use crate::CameraController;

pub struct WaterPlugin;

#[derive(Component)]
struct Water {
    handle: Handle<WaterMaterial>,
}
impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<WaterMaterial>::default());
        app.add_startup_system(
            |mut materials: ResMut<Assets<WaterMaterial>>,
             mut commands: Commands,
             mut meshes: ResMut<Assets<Mesh>>| {
                let material = materials.add(WaterMaterial::default());
                commands.spawn((
                    MaterialMeshBundle {
                        mesh: meshes.add(generate_water_mesh()),
                        material: material.clone(),
                        transform: Transform::from_rotation(Quat::from_euler(
                            EulerRot::XYZ,
                            PI * -0.5,
                            0.,
                            0.,
                        )),
                        ..default()
                    },
                    Water { handle: material },
                ));
            },
        );
        app.add_system(
            |camera: Query<&Transform, With<CameraController>>,
             sun: Query<&Transform, With<DirectionalLight>>,
             water: Query<&Water>,
             mut materials: ResMut<Assets<WaterMaterial>>,
             time: Res<Time>| {
                let camera_position = camera.get_single().unwrap().translation;
                let sun_facing = sun.get_single().unwrap().forward();
                for water in &water {
                    if let Some(material) = materials.get_mut(&water.handle) {
                        material.camera_position = camera_position;
                        material.time = time.raw_elapsed_seconds();
                        material.sun_direction = sun_facing;
                    }
                }
            },
        );
    }
}

fn generate_water_mesh() -> Mesh {
    shape::Circle {
        radius: 90_000.,
        vertices: 200,
    }
    .into()
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone, Default, Reflect)]
#[uuid = "f790fd8e-d598-45ab-8225-97e2a3f056e0"]
pub struct WaterMaterial {
    #[uniform(0)]
    pub sun_direction: Vec3,
    #[uniform(0)]
    pub camera_position: Vec3,
    #[uniform(0)]
    pub scale: Vec3,
    #[uniform(0)]
    pub time: f32,
    // #[texture(1, dimension = "3d")]
    #[texture(1)]
    #[sampler(2)]
    pub noise: Option<Handle<Image>>,
}

impl Material for WaterMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/water.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
