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

/*
#[derive(Clone, Copy, Debug)]
pub struct HeightPosition {
    pub north: f32,
    pub east: f32,
    pub height: f32,
}

impl GroundPosition {
    fn _add_height(&self, height: f32) -> HeightPosition {
        HeightPosition {
            north: self.north,
            east: self.east,
            height,
        }
    }
}
*/

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
    Skillion,
    _Gabled,
    Onion,
    Phyramidal,
}

#[derive(Clone, Debug)]
pub struct BoundingBox {
    pub north: f32,
    pub south: f32,
    pub east: f32,
    pub west: f32,
}

impl BoundingBox {
    pub fn new() -> Self {
        BoundingBox {
            north: f32::MIN,
            south: f32::MAX,
            east: f32::MIN,
            west: f32::MAX,
        }
    }

    pub fn include(&mut self, position: GroundPosition) {
        self.north = self.north.max(position.north);
        self.south = self.south.min(position.north);
        self.east = self.east.max(position.east);
        self.west = self.west.min(position.east);
    }

    pub fn east_larger_than_nord(&self) -> bool {
        self.east - self.west > self.north - self.south
    }
}

// A builiding without parts is its onw part or itselve is a part
#[derive(Clone, Debug)]
pub struct BuildingPart {
    pub _part: bool,
    pub footprint: Vec<GroundPosition>,
    pub center: GroundPosition,
    pub bounding_box: BoundingBox,
    pub wall_height: f32,
    pub min_height: f32,
    pub color: RenderColor,
    pub roof_shape: RoofShape,
    pub roof_height: f32,
    pub roof_angle: f32,
    pub roof_color: RenderColor,
}
