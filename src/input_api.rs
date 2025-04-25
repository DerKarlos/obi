use std::fmt;

pub type ColorAlpha = [f32; 4];

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

pub struct Roof {
    pub shape: Option<String>,
    pub height: Option<f32>,
    pub color: Option<ColorAlpha>,
}

pub struct BuildingOrPart {
    pub _part: bool,
    pub height: Option<f32>,
    pub min_height: Option<f32>,
    pub roof: Option<Roof>,
    pub foodprint: Vec<GroundPosition>,
    pub color: Option<ColorAlpha>,
}
