use bytemuck::{Pod, Zeroable};
use glam::IVec3;
use hyperx_core_world::chunk::{ChunkSection, SECTION_SIZE};
use hyperx_core_world::World;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ChunkVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
    pub color: [f32; 4],
}

pub struct SectionMesh {
    pub vertices: Vec<ChunkVertex>,
    pub indices: Vec<u32>,
}

impl SectionMesh {
    pub fn new() -> Self {
        Self {
            vertices: Vec::with_capacity(1024),
            indices: Vec::with_capacity(1536),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
}

// 6 Directional Face Normals
const NORMALS: [[f32; 3]; 6] = [
    [0.0, 1.0, 0.0],  // Top (Y+)
    [0.0, -1.0, 0.0], // Bottom (Y-)
    [0.0, 0.0, -1.0], // North (Z-)
    [0.0, 0.0, 1.0],  // South (Z+)
    [1.0, 0.0, 0.0],  // East (X+)
    [-1.0, 0.0, 0.0], // West (X-)
];

// Block face quad relative offsets
const FACE_VERTICES: [[[f32; 3]; 4]; 6] = [
    // Top (Y+)
    [[0.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]],
    // Bottom (Y-)
    [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
    // North (Z-)
    [[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]],
    // South (Z+)
    [[0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0]],
    // East (X+)
    [[1.0, 0.0, 1.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [1.0, 1.0, 1.0]],
    // West (X-)
    [[0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 1.0], [0.0, 1.0, 0.0]],
];

const UV_COORDS: [[f32; 2]; 4] = [
    [0.0, 1.0],
    [1.0, 1.0],
    [1.0, 0.0],
    [0.0, 0.0],
];

/// Mesher for one 16x16x16 chunk section
pub fn mesh_section(
    section: &ChunkSection,
    section_origin: IVec3,
    world: &World,
) -> SectionMesh {
    let mut mesh = SectionMesh::new();
    if section.block_count == 0 {
        return mesh;
    }

    let offsets: [IVec3; 6] = [
        IVec3::new(0, 1, 0),
        IVec3::new(0, -1, 0),
        IVec3::new(0, 0, -1),
        IVec3::new(0, 0, 1),
        IVec3::new(1, 0, 0),
        IVec3::new(-1, 0, 0),
    ];

    for y in 0..SECTION_SIZE {
        for z in 0..SECTION_SIZE {
            for x in 0..SECTION_SIZE {
                let block = section.get_block(x, y, z);
                if block == 0 {
                    continue; // Air
                }

                let world_pos = section_origin + IVec3::new(x as i32, y as i32, z as i32);

                // Distinct colors based on block id for visual differentiation
                let color = match block % 5 {
                    1 => [0.28, 0.65, 0.23, 1.0], // Grass green
                    2 => [0.55, 0.40, 0.28, 1.0], // Dirt brown
                    3 => [0.50, 0.50, 0.50, 1.0], // Stone gray
                    4 => [0.65, 0.52, 0.35, 1.0], // Oak wood
                    _ => [0.85, 0.85, 0.60, 1.0], // Sand / other
                };

                for face in 0..6 {
                    let neighbor_pos = world_pos + offsets[face];
                    let neighbor_block = world.get_block(neighbor_pos.x, neighbor_pos.y, neighbor_pos.z);
                    if neighbor_block != 0 {
                        continue; // Culled by adjacent solid block
                    }

                    // Append 4 vertices for this visible face
                    let base_idx = mesh.vertices.len() as u32;
                    let face_v = &FACE_VERTICES[face];
                    let normal = NORMALS[face];

                    for v in 0..4 {
                        let pos = [
                            world_pos.x as f32 + face_v[v][0],
                            world_pos.y as f32 + face_v[v][1],
                            world_pos.z as f32 + face_v[v][2],
                        ];
                        mesh.vertices.push(ChunkVertex {
                            position: pos,
                            uv: UV_COORDS[v],
                            normal,
                            color,
                        });
                    }

                    // 2 triangles: 0, 1, 2 and 2, 3, 0
                    mesh.indices.push(base_idx);
                    mesh.indices.push(base_idx + 1);
                    mesh.indices.push(base_idx + 2);
                    mesh.indices.push(base_idx + 2);
                    mesh.indices.push(base_idx + 3);
                    mesh.indices.push(base_idx);
                }
            }
        }
    }

    mesh
}
