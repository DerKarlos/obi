// Internal Interface of the crate/lib between input modules/crates and a renderer

use i_float::float::compatible::FloatPointCompatible;
use serde::Deserialize;
use std::collections::HashMap;

pub static LAT_FAKT: f64 = 111100.0; // 111285; // exactly enough  111120 = 1.852 * 1000.0 * 60  // 1 NM je Bogenminute: 1 Grad Lat = 60 NM = 111 km, 0.001 Grad = 111 m
pub static PI: f32 = std::f32::consts::PI;

//use std::fmt::Display;
use std::fmt;
use std::ops::{Add, Sub};

use crate::footprint::Footprint;

#[derive(Default, Clone, Copy, Debug)]
pub struct GeographicCoordinates {
    pub latitude: f64,
    pub longitude: f64,
}

impl GeographicCoordinates {
    pub fn coordinates_to_position(&self, latitude: f64, longitude: f64) -> GroundPosition {
        // What s that vor ???
        if self.latitude == 0. {
            return GroundPosition {
                north: latitude as f32,
                east: longitude as f32,
            };
        }

        // the closer to the pole, the smaller the tiles size in meters get
        let lon_fakt = LAT_FAKT * ((latitude / 180. * PI as f64).abs()).cos();
        // Longitude(Längengrad) West/East factor
        // actual coor - other coor = relative grad/meter ground position

        GroundPosition {
            north: ((latitude - self.latitude) * LAT_FAKT) as f32,
            east: ((longitude - self.longitude) * lon_fakt) as f32,
        }
    }
}

// See for standard 2D features like Add: https://docs.rs/vector2/latest/vector2/struct.Vector2.html
#[derive(Debug, Clone, Copy)]
pub struct GroundPosition {
    pub north: f32,
    pub east: f32,
}

impl Default for GroundPosition {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Add for GroundPosition {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            north: self.north + other.north,
            east: self.east + other.east,
        }
    }
}

impl Sub for GroundPosition {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            north: self.north - other.north,
            east: self.east - other.east,
        }
    }
}

// Implement the `FloatPointCompatible` trait for CustomPoint
impl FloatPointCompatible<f32> for GroundPosition {
    fn from_xy(x: f32, y: f32) -> Self {
        Self { east: x, north: y }
    }

    fn x(&self) -> f32 {
        self.east
    }

    fn y(&self) -> f32 {
        self.north
    }
}

impl GroundPosition {
    /// Shorthand for writing `Vector2::new(0.0, 0.0)`.
    pub const ZERO: Self = Self {
        north: 0.0,
        east: 0.0,
    };

    pub fn distance_angle_to_other(&self, other: &GroundPosition) -> (f32, f32) {
        let a = self.north - other.north;
        let b = self.east - other.east;
        let distance = f32::sqrt(a * a + b * b);

        // Its atan2(y,x)   NOT:x,y!
        // East = (0,1) = 0    Nord(1,0) = 1.5(Pi/2)   West(0,-1) = 3,14(Pi)   South(-1,0) = -1.5(-Pi)
        let angle: f32 = f32::atan2(other.east - self.east, other.north - self.north);
        // why - negativ??? (see other lines)
        //let angle: f32 = f32::atan2(self.east - other.east, self.north - other.north);

        (distance, angle)
    }

    pub fn rotate(self, angle: f32) -> GroundPosition {
        let cos = f32::cos(angle);
        let sin = f32::sin(angle);
        // Don't change this lines! They are correct and tested. If something is odd, look on your code, calling rotate()
        let north = -sin * self.east + cos * self.north;
        let east = cos * self.east + sin * self.north;
        //println!("angle: {angle} sin: {sin} cos: {cos} sn: {} se: {} n: {} e: {}",self.north, self.east, north, east);

        GroundPosition { north, east }
    }
}

impl std::fmt::Display for GroundPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.east, self.north)
    }
}

// todo?: move to (ALL?) input_osm_*
pub struct OtbNode {
    pub position: GroundPosition,
}

#[derive(Debug, Clone)]
pub struct OtbWay {
    pub _id: u64,
    pub footprint: Footprint,
    pub tags: Option<HashMap<String, String>>,
}

#[derive(Debug)]
pub struct OtbRelation {
    pub id: u64,
    pub members: Vec<Member>,
    pub tags: Option<HashMap<String, String>>,
}

// Internal type of the 3d-renderer. It's just luck, it is the same as needed for the gpu-renderer Bevy ;-)
pub type RenderColor = [f32; 4];

#[derive(Clone, Copy, Debug)]
pub enum RoofShape {
    None,
    _Unknown,
    Flat,
    Skillion,
    Gabled,
    Phyramidal,
    Dome,
    Onion,
}

#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub north: f32,
    pub south: f32,
    pub east: f32,
    pub west: f32,
}

impl fmt::Display for BoundingBox {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{},{},{},{}",
            self.west, self.south, self.east, self.north
        )
        //write!(f, "I am A")
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn max_radius(&self) -> f64 {
        (self.east as f64 - self.west as f64).max(self.north as f64 - self.south as f64)
    }

    pub fn center_as_geographic_coordinates(&self) -> GeographicCoordinates {
        let latitude = (self.south + (self.north - self.south) / 2.) as f64;
        let longitude = (self.west + (self.east - self.west) / 2.) as f64;

        GeographicCoordinates {
            latitude,
            longitude,
        }
    }

    pub fn include(&mut self, position: &GroundPosition) {
        self.north = self.north.max(position.north);
        self.south = self.south.min(position.north);
        self.east = self.east.max(position.east);
        self.west = self.west.min(position.east);
    }

    pub fn min_range(&mut self, range: f32) {
        println!("{self}");
        // range in meter to degres
        let range = range / LAT_FAKT as f32;
        let center_north = (self.north - self.south) / 2. + self.south;
        let center_east = (self.east - self.west) / 2. + self.west;
        println!("{range} {center_north} {center_east}");
        self.north = self.north.max(center_north + range);
        self.south = self.south.min(center_north - range);
        self.east = self.east.max(center_east + range);
        self.west = self.west.min(center_east - range);
        println!("{self}");
    }

    pub fn shift(&mut self, shift: f32) {
        self.north += shift;
        self.south += shift;
        self.east += shift;
        self.west += shift;
    }
}

// A builiding without parts is its onw part or itselve is a part
#[derive(Clone, Debug)]
pub struct BuildingPart {
    pub id: u64,
    pub part: bool,
    pub footprint: Footprint,
    //pub _bounding_box: BoundingBox,
    pub bounding_box_rotated: BoundingBox,
    // upper heit of the wall, independend of / including the min_height
    pub wall_height: f32,
    pub min_height: f32,
    pub building_color: RenderColor,
    pub roof_shape: RoofShape,
    pub roof_height: f32,
    pub roof_angle: f32,
    pub roof_color: RenderColor,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Member {
    #[serde(rename = "type")]
    pub relation_type: String,
    #[serde(rename = "ref")]
    pub reference: u64,
    #[serde(rename = "role")]
    pub role: String,
}
