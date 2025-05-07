// Internal Interface of the crate/lib between a renderer and output modules/crates

// Internal type of the 3d-renderer. It's just luck, it is the same as needed for the gpu-renderer Bevy ;-)
pub type RenderColor = [f32; 4];

pub type GpuPosition = [f32; 3];

// Mesh render attributes (may be mor later)
pub struct OsmMeshAttributes {
    pub indices_to_vertices: Vec<u32>,
    pub vertices_colors: Vec<RenderColor>, // format: Float32x4
    pub vertices_positions: Vec<GpuPosition>, // 3 coordinates * x Positions. The corners are NOT reused to get hard Kanten
                                              // todo?: not pub but fn get
}

impl OsmMeshAttributes {
    pub fn new() -> Self {
        Self {
            indices_to_vertices: vec![],
            vertices_colors: vec![],
            vertices_positions: vec![],
        }
    }
}
