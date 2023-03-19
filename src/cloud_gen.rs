use crate::sdf::sdf as cloud_sdf;
use bevy::math::vec3;
use bevy::prelude::Mesh;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use fast_surface_nets::ndshape::{ConstShape, ConstShape3u32};
use fast_surface_nets::{surface_nets, SurfaceNetsBuffer};

// A 16^3 chunk with 1-voxel boundary padding.
const RES: f32 = 200.0;
type ChunkShape = ConstShape3u32<{ RES as u32 }, { RES as u32 }, { RES as u32 }>;

// This chunk will cover just a single octant of a sphere SDF (radius 15).
pub fn test() -> Mesh {
    let mut sdf = vec![1.0; ChunkShape::USIZE];
    for i in 0u32..ChunkShape::SIZE {
        let [x, y, z] = ChunkShape::delinearize(i);
        let p = (vec3(x as f32, y as f32, z as f32) / RES - 0.5) * 2.;
        let d = cloud_sdf(p);
        // println!("{p}, {d}");
        sdf[i as usize] = d;
    }

    let mut buffer = Box::new(SurfaceNetsBuffer::default());
    surface_nets(
        &sdf,
        &ChunkShape {},
        [0; 3],
        [RES as u32 - 1; 3],
        &mut buffer,
    );

    // println!("{:#?}", sdf);
    println!("{:#?}", buffer.positions.len());
    println!(
        "{:?}",
        sdf.iter()
            .fold(100., |prev, next| if next < &prev { *next } else { prev })
    );
    println!(
        "{:?}",
        sdf.iter()
            .fold(-100., |prev, next| if next > &prev { *next } else { prev })
    );
    // Some triangles were generated.
    assert!(!buffer.indices.is_empty());
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U32(buffer.indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, buffer.positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, buffer.normals);
    mesh
}
