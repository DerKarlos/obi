// Internal Interface of the crate/lib between input modules/crates and a renderer

#[derive(Clone, Copy, Debug)]
pub struct GeographicCoordinates {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct GroundPosition {
    pub north: f32,
    pub east: f32,
}

impl std::fmt::Display for GroundPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.east, self.north)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct OsmNode {
    pub position: GroundPosition,
}

// Internal type of the 3d-renderer. It's just luck, it is the same as needed for the gpu-renderer Bevy ;-)
pub type RenderColor = [f32; 4];

#[derive(Clone, Copy, Debug)]
pub enum RoofShape {
    None,
    Unknown,
    Flat,
    Onion,
    Phyramidal,
}

#[derive(Clone, Copy, Debug)]
pub struct Roof {
    pub shape: RoofShape,
    pub height: Option<f32>,
    pub color: Option<RenderColor>,
}

#[derive(Clone, Debug)]
pub struct BuildingOrPart {
    pub _part: bool,
    pub footprint: Vec<GroundPosition>,
    pub _longest_side_index: u32,
    pub _center: GroundPosition,
    pub height: Option<f32>,
    pub min_height: Option<f32>,
    pub roof: Option<Roof>,
    pub color: Option<RenderColor>,
}
