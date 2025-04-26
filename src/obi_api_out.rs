// Internal Interface of OBI between a renderer and output modules/crates

// Internal type of the 3d-renderer. It's just luck, it is the same as needed for the gpu-renderer Bevy ;-)
pub type RenderColor = [f32; 4];

pub type GpuPosition = [f32; 3];

// Mesh render attributes (may be mor later)
pub struct OsmMesh {
    pub vertices_colors: Vec<RenderColor>,    // format: Float32x4
    pub vertices_positions: Vec<GpuPosition>, // 3 coordinates * x Positions. The corners are NOT reused to get hard Kanten
    pub indices_to_vertices: Vec<u32>,
    // todo?: not pub but fn get
}

impl OsmMesh {
    pub fn new() -> Self {
        Self {
            vertices_colors: vec![],
            vertices_positions: vec![],
            indices_to_vertices: vec![],
        }
    }
}
