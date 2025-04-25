// todo: split in input_api and output_api

use std::fmt;

#[derive(Clone, Copy, Debug)]
pub struct GeographicCoordinates {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct GroundPosition {
    pub east: f32,
    pub north: f32,
}

impl fmt::Display for GroundPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.east, self.north)
    }
}

pub struct OsmNode {
    pub position: GroundPosition,
}

// Internal type of the renderer. It's just luck, it is the same as needed for Bevy ;-)
pub type RenderColor = [f32; 4];

#[derive(Debug)]
pub enum RoofShape {
    None,
    Unknown,
    Flat,
    Onion,
    Phyramidal,
}

pub struct Roof {
    pub shape: RoofShape,
    pub height: Option<f32>,
    pub color: Option<RenderColor>,
}

pub struct BuildingOrPart {
    pub _part: bool,
    pub height: Option<f32>,
    pub min_height: Option<f32>,
    pub roof: Option<Roof>,
    pub foodprint: Vec<GroundPosition>,
    pub color: Option<RenderColor>,
}
