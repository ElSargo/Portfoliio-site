use crate::{noise, CameraController};
use bevy::{
    math::{dvec2, dvec3, ivec3, vec2, vec3, vec4, DVec2, DVec3},
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{Indices, MeshVertexAttribute, MeshVertexBufferLayout, VertexAttributeValues},
        render_resource::{
            AsBindGroup, Extent3d, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError, TextureDimension, TextureFormat,
        },
    },
    utils::{HashMap, HashSet},
};
use itertools::Itertools;

/*
Clouds that have simple geometry that aproxiates thier shape, and have "fins"
that extrude from the edges of the mesh, that have opacity based on noise.

All the info for density and self-shadows is baked into a 2D texture and sampled
using the mesh uv, the cloud desnsity is the distance to the mesh displaced by
the cell-value-fbm.
*/

pub struct FinCloudPlugin;
impl Plugin for FinCloudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<FinCloudMaterial>::default());
        app.add_system(update_cloud);
        app.add_startup_system(setup);
    }
}

fn update_cloud(
    camera: Query<&Transform, With<CameraController>>,
    sun: Query<&Transform, With<DirectionalLight>>,
    clouds: Query<&FinCloudBase>,
    mut materials: ResMut<Assets<FinCloudMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let camera = camera.get_single().unwrap();
    let view = camera.forward();
    let camera_position = camera.translation;
    let sun_facing = sun.get_single().unwrap().forward();
    for fin_cloud in &clouds {
        if let Some(mesh) = meshes.get_mut(&fin_cloud.mesh) {
            let iview = view.as_ivec3();
            let sign_with_zeros = iview.signum();
            let one_if_zero = |x| if x == 0 { 1 } else { x };
            let sign = ivec3(
                one_if_zero(sign_with_zeros.x),
                one_if_zero(sign_with_zeros.y),
                one_if_zero(sign_with_zeros.z),
            );
            let mut weight = [2; 3];
            let abs = view.abs();
            let max = abs.max_element();
            let min = abs.min_element();
            let mut max_found = false;
            let mut min_found = false;
            for (i, v) in abs.to_array().into_iter().enumerate() {
                if v == max && !max_found {
                    max_found = true;
                    weight[i] = 3;
                }
                if v == min && !min_found {
                    min_found = true;
                    weight[i] = 1;
                }
            }
            let key = sign * IVec3::from(weight);
            mesh.set_indices(Some(Indices::U32(fin_cloud.indices[&key].clone())))
        }
        if let Some(material) = materials.get_mut(&fin_cloud.material) {
            material.camera_position = camera_position;
            material.sun_direction = sun_facing;
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FinCloudMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let base_mesh_data = extract_mesh_data(
        shape::UVSphere {
            radius: 1.,
            sectors: 8,
            stacks: 4,
        }
        .into(),
    )
    .expect("Extraction failed");
    let resoluiton = (2048, 2048);
    let new_mesh_data = generate_fin_data(&base_mesh_data, 1., resoluiton);
    let sorted_indices = voluetric_sort_and_cull(&new_mesh_data.indices, &new_mesh_data.positions);
    let position_texture = rasterize_uv(&new_mesh_data, base_mesh_data.indices.len(), resoluiton);
    let cloud_texture = generate_cloud_texture(&new_mesh_data, &position_texture, resoluiton);
    let mesh = meshes.add(new_mesh_data.into());
    let material = materials.add(FinCloudMaterial {
        texture: Some(
            images.add(Image::new(
                Extent3d {
                    width: resoluiton.1 as u32,
                    height: resoluiton.0 as u32,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                cloud_texture
                    .iter()
                    .flatten()
                    .flat_map(|v| {
                        [
                            v.x.to_ne_bytes(),
                            v.y.to_ne_bytes(),
                            v.z.to_ne_bytes(),
                            v.w.to_ne_bytes(),
                        ]
                    })
                    .flatten()
                    .collect(),
                TextureFormat::Rgba32Float,
            )),
        ),
        ..default()
    });
    commands
        .spawn((
            FinCloudBase {
                material: material.clone(),
                indices: sorted_indices,
                mesh: mesh.clone(),
            },
            MaterialMeshBundle {
                mesh,
                material,
                ..default()
            },
        ))
        .push_children(&[commands
            .spawn((
                FinCloudFin {
                    material: material.clone(),
                    indices: sorted_indices,
                    mesh: mesh.clone(),
                },
                MaterialMeshBundle {
                    mesh,
                    material,
                    ..default()
                },
            ))
            .id()]);
}

fn generate_cloud_texture(
    _meshdata: &MeshData,
    position_texture: &[Vec<(DVec3, f64)>],
    resolution: (usize, usize),
) -> Vec<Vec<Vec4>> {
    let mut data = vec![vec![Vec4::ZERO; resolution.1]; resolution.0];
    for (cell, (position, transparency_signal)) in data
        .iter_mut()
        .flatten()
        .zip(position_texture.iter().flatten())
    {
        *cell = vec4(
            noise::wfbm(
                vec3(
                    (position.x * 4.) as f32,
                    (position.y * 4.) as f32,
                    (position.z * 4.) as f32,
                ),
                vec3(1000., 1000., 1000.),
            ),
            *transparency_signal as f32,
            (position.y / position.length() * 1.5 + 1.) as f32,
            1.,
        );
    }
    data
}

fn extract_mesh_data(mesh: Mesh) -> Option<MeshData> {
    let get_3 = |atr: MeshVertexAttribute| {
        Some(
            mesh.attribute(atr.id)?
                .as_float3()?
                .into_iter()
                .map(|[x, y, z]| DVec3 {
                    x: *x as f64,
                    y: *y as f64,
                    z: *z as f64,
                })
                .collect(),
        )
    };
    let positions: Vec<DVec3> = get_3(Mesh::ATTRIBUTE_POSITION)?;
    let normals = get_3(Mesh::ATTRIBUTE_NORMAL)?;

    let uvs: Vec<DVec2> = match mesh.attribute(Mesh::ATTRIBUTE_UV_0.id)? {
        VertexAttributeValues::Float32x2(vec) => vec,
        _ => unimplemented!(),
    }
    .into_iter()
    .map(|[x, y]| DVec2 {
        x: *x as f64,
        y: *y as f64,
    })
    .collect();

    let indices = match mesh.indices()? {
        Indices::U32(data) => data,
        Indices::U16(_) => unimplemented!(),
    }
    .to_owned();

    if positions.len() != normals.len() || normals.len() != uvs.len() {
        return None;
    }

    Some(MeshData {
        positions,
        normals,
        uvs,
        indices,
    })
}

fn generate_fin_data(data: &MeshData, scale: f64, resolution: (usize, usize)) -> FinData {
    let num_indices = data.indices.len();
    let num_vertices = data.positions.len();
    let new_buffer_size = num_vertices + num_indices * 9 / 2;
    let mut indices = Vec::with_capacity(num_indices * 3);
    let mut positions = Vec::with_capacity(new_buffer_size);
    let mut uvs = Vec::with_capacity(new_buffer_size);
    // Bool flag to signal a vertex on the transparent edge

    /*
    A fin is a square extruded out from an edge on the mesh
      [+++++] // Each fin has 2 triangles and four vertices.
      *1---3  // No vertices are shared inorder to asign uv's.
     /| \  |  // Vertiecs 1 and two are in the same position
    * |  \ |  // as the edge veritecs.
     \|   \|  // 3 and four are displaced out in the direction
      *2---4  // of the normal at 1 and 2 respectively.
      [+++++] // The height is controled by the scale parameter
    */
    let mut fin_positions = Vec::new();
    for (position, normal) in data.positions.iter().zip(data.normals.iter()) {
        fin_positions.push(*position);
        fin_positions.push(*position + *normal * scale);
        // y will be set later
    }

    /*
    Iterate over the edges of the mesh and create the corosponding fin
    Edges are defined by indesices which are aranged in terms of triangles,
    therfore we shall loop the triangles, make egdes and remove duplicates.
      2
     /|
    1 | -> [(1,2),(2,3),(1,3)]
     \|
      3
    */
    let mut found = HashSet::with_capacity(num_indices / 2);
    for i in (0..num_indices / 3).map(|i| i * 3) {
        let a = data.indices[i];
        let b = data.indices[i + 1];
        let c = data.indices[i + 2];

        // Sort indices so duplacates can be removed
        for edge in [
            (a.min(b), a.max(b)),
            (b.min(c), b.max(c)),
            (a.min(c), a.max(c)),
        ] {
            found.insert(edge);
        }
        // Next triangle
    }
    for (i, edge) in found.iter().enumerate() {
        //0a-b
        // |\|
        //1c-d
        // [oooooooo|11,22,33,44,55]
        // Push fin vertices
        positions.extend_from_slice(&[
            fin_positions[(edge.0 * 2) as usize],
            fin_positions[(edge.0 * 2) as usize + 1],
            fin_positions[(edge.1 * 2) as usize],
            fin_positions[(edge.1 * 2) as usize + 1],
        ]);
        uvs.extend_from_slice(&[
            dvec3(0.0, 1.0, i as f64),
            dvec3(1.0, 1.0, i as f64),
            dvec3(0.0, 0.0, i as f64),
            dvec3(1.0, 0.0, i as f64),
        ]);
        let i_vert = i * 4;
        let a = (i_vert) as u32;
        let b = (i_vert + 1) as u32;
        let c = (i_vert + 2) as u32;
        let d = (i_vert + 3) as u32;

        indices.extend_from_slice(&[a, d, b]);
        indices.extend_from_slice(&[a, c, d]);
        i_vert += 4;
    }

    let num_fins = (positions.len() / 4) as f64;
    uvs.iter_mut().map(|z| z /= num_fins);

    FinData {
        positions,
        uvs,
        indices,
    }
}

struct FinData {
    positions: Vec<DVec3>,
    uvs: Vec<DVec3>,
    indices: Vec<u32>,
}

struct Triangle {
    mid_point: DVec3,
    indices: [u32; 3],
}

//https://iquilezles.org/articles/volumesort/
fn voluetric_sort_and_cull(indices: &[u32], positions: &[DVec3]) -> HashMap<IVec3, Vec<u32>> {
    let mut triangles = Vec::with_capacity(indices.len() / 3);
    for i in (0..indices.len() / 3).map(|i| i * 3) {
        let (a, b, c) = (indices[i], indices[i + 1], indices[i + 2]);
        let (pa, pb, pc) = (
            positions[a as usize],
            positions[b as usize],
            positions[c as usize],
        );
        triangles.push(Triangle {
            mid_point: (pa + pb + pc) / 3.,
            indices: [a, b, c],
        });
    }
    let mut sorted_indices = HashMap::with_capacity(48);
    let wieghts = [1, 2, 3];
    let flags = [1, -1];
    for (wieght, direction) in wieghts.iter().permutations(3).cartesian_product(
        flags
            .iter()
            .cartesian_product(flags.iter())
            .cartesian_product(flags.iter())
            .map(|tupple| [*tupple.0 .0, *tupple.0 .1, *tupple.1]),
    ) {
        let key = ivec3(
            wieght[0] * direction[0],
            wieght[1] * direction[1],
            wieght[2] * direction[2],
        );
        let point = (key.as_dvec3() / 3.).normalize();

        // let mut visable_triangles = triangles
        //     .iter()
        //     .flat_map(|t| {
        //         let a = positions[t.indices[0] as usize];
        //         let b = positions[t.indices[1] as usize];
        //         let normal = (a - t.mid_point)
        //             .normalize()
        //             .cross((b - t.mid_point).normalize());
        //         if point.dot(normal).abs() < 0.2 {
        //             None
        //         } else {
        //             Some(t)
        //         }
        //     })
        //     .collect_vec();

        triangles.sort_unstable_by_key(|triangle| {
            ordered_float::OrderedFloat(-triangle.mid_point.dot(point))
        });
        sorted_indices.insert(
            key,
            triangles
                .iter()
                .flat_map(|t| t.indices)
                .collect::<Vec<u32>>(),
        );
    }
    sorted_indices
}

fn rasterize_uv(
    mesh: &MeshData,
    edge_offset: usize,
    resolution: (usize, usize),
) -> Vec<Vec<(DVec3, f64)>> {
    // Iterate triangles
    let mut i = 0;
    let mut texture = vec![vec![(DVec3::ONE, 1.); resolution.1]; resolution.0];
    // Iterate triangles of solid geometry
    while i < edge_offset {
        let a_i = mesh.indices[i] as usize;
        let b_i = mesh.indices[i + 1] as usize;
        let c_i = mesh.indices[i + 2] as usize;
        let a_position = mesh.positions[a_i];
        let b_position = mesh.positions[b_i];
        let c_position = mesh.positions[c_i];
        let a_uv = mesh.uvs[a_i];
        let b_uv = mesh.uvs[b_i];
        let c_uv = mesh.uvs[c_i];
        let scale = dvec2(resolution.0 as f64, resolution.1 as f64);
        let a = a_uv * scale;
        let b = b_uv * scale;
        let c = c_uv * scale;

        raster(a, b, c, |x| {
            let t = bary_lerp(x, a, b, c);
            let position = a_position * t.x + b_position * t.y + c_position * t.z;

            if let Some(row) = texture.get_mut(x.y as usize) {
                if let Some(pix) = row.get_mut(x.x as usize) {
                    *pix = (position, 1.);
                }
            }
        });

        i += 3;
    }
    // Iterate over quads (as two triangles) to provide transparency at horozontal edges
    while i < mesh.indices.len() {
        let a_i = mesh.indices[i] as usize;
        let b_i = mesh.indices[i + 1] as usize;
        let c_i = mesh.indices[i + 2] as usize;
        let d_i = mesh.indices[i + 3] as usize;
        let e_i = mesh.indices[i + 4] as usize;
        let f_i = mesh.indices[i + 5] as usize;
        let a_position = mesh.positions[a_i];
        let b_position = mesh.positions[b_i];
        let c_position = mesh.positions[c_i];
        let d_position = mesh.positions[d_i];
        let e_position = mesh.positions[e_i];
        let f_position = mesh.positions[f_i];
        let a_uv = mesh.uvs[a_i];
        let b_uv = mesh.uvs[b_i];
        let c_uv = mesh.uvs[c_i];
        let d_uv = mesh.uvs[d_i];
        let e_uv = mesh.uvs[e_i];
        let f_uv = mesh.uvs[f_i];
        let scale = dvec2(resolution.0 as f64, resolution.1 as f64);
        let a = a_uv * scale;
        let b = b_uv * scale;
        let c = c_uv * scale;
        let d = d_uv * scale;
        let e = e_uv * scale;
        let f = f_uv * scale;

        //0a-b
        // |\|
        //1c-d
        // new_indices.extend_from_slice(&[a, d, b]);
        // new_indices.extend_from_slice(&[a, c, d]);
        raster(a, b, c, |x| {
            let t = bary_lerp(x, a, b, c);
            let position = a_position * t.x + b_position * t.y + c_position * t.z;

            if let Some(row) = texture.get_mut(x.y as usize) {
                if let Some(pix) = row.get_mut(x.x as usize) {
                    *pix = (position, sd_line(x, a, c).min(sd_line(x, e, f)));
                }
            }
        });

        raster(d, e, f, |x| {
            let t = bary_lerp(x, d, e, f);
            let position = d_position * t.x + e_position * t.y + f_position * t.z;

            if let Some(row) = texture.get_mut(x.y as usize) {
                if let Some(pix) = row.get_mut(x.x as usize) {
                    *pix = (position, sd_line(x, a, c).min(sd_line(x, e, f)));
                }
            }
        });

        i += 6;
    }

    // println!(
    //     "{} not written out of {}",
    //     texture
    //         .iter()
    //         .flatten()
    //         .filter(|v| **v == DVec3::ONE)
    //         .count(),
    //     resolution.0 * resolution.1
    // );

    texture
}

fn bary_lerp(t: DVec2, a: DVec2, b: DVec2, c: DVec2) -> DVec3 {
    let alpha = ((b.y - c.y) * (t.x - c.x) + (c.x - b.x) * (t.y - c.y))
        / ((b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y));
    let beta = ((c.y - a.y) * (t.x - c.x) + (a.x - c.x) * (t.y - c.y))
        / ((b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y));
    dvec3(alpha, beta, 1. - alpha - beta)
}

//https://iquilezles.org/articles/distfunctions2d/
fn sd_line(p: DVec2, a: DVec2, b: DVec2) -> f64 {
    let pa = p - a;
    let ba = b - a;
    let h = (pa.dot(ba) / ba.dot(ba)).clamp(0.0, 1.0);
    (pa - ba * h).length()
}

//https://iquilezles.org/articles/distfunctions2d/
fn sd_triangle(p: DVec2, p0: DVec2, p1: DVec2, p2: DVec2) -> f64 {
    let e0 = p1 - p0;
    let e1 = p2 - p1;
    let e2 = p0 - p2;
    let v0 = p - p0;
    let v1 = p - p1;
    let v2 = p - p2;
    let pq0 = v0 - e0 * (v0.dot(e0) / e0.dot(e0)).clamp(0.0, 1.0);
    let pq1 = v1 - e1 * (v1.dot(e1) / e1.dot(e1)).clamp(0.0, 1.0);
    let pq2 = v2 - e2 * (v2.dot(e2) / e2.dot(e2)).clamp(0.0, 1.0);
    let s = (e0.x * e2.y - e0.y * e2.x).signum();
    let d = dvec2(pq0.dot(pq0), s * (v0.x * e0.y - v0.y * e0.x))
        .min(dvec2(pq1.dot(pq1), s * (v1.x * e1.y - v1.y * e1.x)))
        .min(dvec2(pq2.dot(pq2), s * (v2.x * e2.y - v2.y * e2.x)));
    return -(d.x).sqrt() * (d.y).signum();
}

fn raster<F: FnMut(DVec2)>(a: DVec2, b: DVec2, c: DVec2, mut f: F) {
    let lower = dvec2(a.x.min(b.x).min(c.x), a.y.min(b.y).min(c.y));
    let upper = dvec2(a.x.max(b.x).max(c.x), a.y.max(b.y).max(c.y));
    let mut x = lower.x;
    let mut y = lower.y;
    while x < upper.x {
        while y < upper.y {
            if sd_triangle(dvec2(x, y), a, b, c) <= 1. {
                f(dvec2(x, y));
            }
            y += 1.;
        }
        x += 1.;
        y = lower.y;
    }
}

#[derive(Component, Default)]
struct FinCloudBase {
    material: Handle<FinCloudMaterial>,
    mesh: Handle<Mesh>,
    indices: HashMap<IVec3, Vec<u32>>,
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone, Default, Reflect)]
#[uuid = "f690fd8e-d598-46ab-8225-97e2a3f056e0"]
pub struct FinCloudMaterial {
    #[uniform(0)]
    sun_direction: Vec3,
    #[uniform(0)]
    camera_position: Vec3,
    #[texture(1)]
    #[sampler(2)]
    texture: Option<Handle<Image>>,
}

impl Material for FinCloudMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/fin_cloud.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        // match &mut descriptor.depth_stencil {
        //     Some(stencil) => stencil.depth_write_enabled = true,
        //     None => println!("No sten"),
        // }
        Ok(())
    }
}

struct MeshData {
    positions: Vec<DVec3>,
    normals: Vec<DVec3>,
    uvs: Vec<DVec2>,
    indices: Vec<u32>,
}

fn vec_dvec3_to_vec_vec3(v: &[DVec3]) -> Vec<Vec3> {
    v.iter()
        .map(|dv| vec3(dv.x as f32, dv.y as f32, dv.z as f32))
        .collect_vec()
}
impl From<MeshData> for Mesh {
    fn from(value: MeshData) -> Self {
        let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec_dvec3_to_vec_vec3(&value.positions),
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            vec_dvec3_to_vec_vec3(&value.normals),
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            value
                .uvs
                .iter()
                .map(|dv| vec2(dv.x as f32, dv.y as f32))
                .collect_vec(),
        );
        mesh.set_indices(Some(Indices::U32(value.indices)));
        mesh
    }
}
