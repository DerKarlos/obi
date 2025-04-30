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
    _Unknown,
    Flat,
    _Skillion,
    _Gabled,
    Onion,
    Phyramidal,
}

// A builiding without parts is its onw part or itselve is a part
#[derive(Clone, Debug)]
pub struct BuildingPart {
    pub _part: bool,
    pub footprint: Vec<GroundPosition>,
    pub _longest_side_index: u32,
    pub center: GroundPosition,
    pub wall_height: f32,
    pub min_height: f32,
    pub color: RenderColor,
    pub roof_shape: RoofShape,
    pub roof_height: f32,
    pub roof_color: RenderColor,
}
