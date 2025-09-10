// Internal Interface of the crate/lib between input modules/crates and a renderer

use std::collections::HashMap;

use serde::Deserialize;

pub static PI: f32 = std::f32::consts::PI;
pub static LAT_FAKT: f64 = 111120.0; // 111100.0  111285; // exactly enough  111120 = 1.852 * 1000.0 * 60 - It is in the OSM wiki: 1′ = 1.852 km * 60s/min * 1000m/km = 111120m

use crate::footprint::Footprint;

#[derive(Default, Clone, Copy, Debug)]
pub struct GeographicCoordinates {
    pub latitude: f64,
    pub longitude: f64,
}

impl GeographicCoordinates {
    /*
     * Rotate lat/lon to reposition the home point onto 0,0.
     * @param {[number, number]} lonLat - The longitute and latitude of a point.
     * @return {[number, number]} x, y in meters
     */

    pub fn coordinates_to_position(&self, latitude: f64, longitude: f64) -> GroundPosition {
        // If no GPU 0 position is set, return just the GPS position. Used to find the GPU 0 position
        if self.latitude == 0. {
            return GroundPosition {
                x: longitude,
                y: latitude,
            };
        }

        // The closer to a pole, the smaller the tiles size in meters get
        let lon_fakt = LAT_FAKT * ((latitude / 180. * PI as f64).abs()).cos();
        // Longitude(Längengrad) West/East factor

        GroundPosition {
            y: ((latitude - self.latitude) * LAT_FAKT),
            x: ((longitude - self.longitude) * lon_fakt),
        }
    }
}

/*************************************
**************************************/

pub type GroundPosition = geo::Coord;
pub type GroundPositions = Vec<geo::Coord>;
pub type OsmMap = HashMap<String, String>;

/********* /
// See for standard 2D features like Add: https://docs.rs/vector2/latest/vector2/struct.Vector2.html
#[derive(Debug, Clone, Copy)]
pub struct _GroundPosition {
    pub y: f64,
    pub x: f64,
}

impl Default for _GroundPosition {
    fn default() -> Self {
        Self::ZERO
    }
}

impl _GroundPosition {
    /// Shorthand for writing `Vector2::new(0.0, 0.0)`.
    pub const ZERO: Self = Self { y: 0.0, x: 0.0 };

    pub fn to_coord(&self) -> geo::Coord {
        geo::Coord {
            x: self.x,
            y: self.y,
        }
    }

    pub fn to_point(&self) -> geo::Point {
        geo::Point::new(self.x, self.y)
    }

    pub fn distance_angle_to_other(&self, other: &_GroundPosition) -> (f64, f64) {
        let a = self.y - other.y;
        let b = self.x - other.x;
        let distance = f64::sqrt(a * a + b * b);

        // Its atan2(y,x)   NOT:x,y!
        // East = (0,1) = 0    Nord(1,0) = 1.5(Pi/2)   West(0,-1) = 3,14(Pi)   South(-1,0) = -1.5(-Pi)
        let angle: f64 = f64::atan2(other.x - self.x, other.y - self.y);
        // why - negativ??? (see other lines)
        //let angle: f32 = f32::atan2(self.east - other.east, self.north - other.north);

        (distance, angle)
    }

    pub fn rotate(self, angle: f32) -> _GroundPosition {
        let cos = FGP::cos(angle as FGP);
        let sin = FGP::sin(angle as FGP);
        // Don't change this lines! They are correct and tested. If something is odd, look on your code, calling rotate()
        let north = -sin * self.x + cos * self.y;
        let east = cos * self.x + sin * self.y;
        //println!("angle: {angle} sin: {sin} cos: {cos} sn: {} se: {} n: {} e: {}",self.north, self.east, north, east);

        _GroundPosition { y: north, x: east }
    }
}

pub type GroundPositions = Vec<GroundPosition>;
pub type Polygon = Vec<GroundPositions>;
pub type Polygons = Vec<Polygon>;

pub const FIRST_POLYGON: usize = 0;
pub const OUTER_POLYGON: usize = 0;
pub const FIRST_HOLE_INDEX: usize = 1;

impl std::fmt::Display for _GroundPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
/ *********/

// todo?: move to (ALL?) input_osm_*

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

/*
 * Extend the area of the OSM object to the given range at last
 * @param {f32} range in meters - the minimum range of the bounding box
 */
pub fn max_range(rect: &mut geo::Rect, range: f64) {
    // range in meter to degres
    let range = range / LAT_FAKT;
    let center = rect.center();
    //println!("{range} {center}");
    rect.set_max(geo::Coord {
        x: rect.max().x.max(center.x + range),
        y: rect.max().y.max(center.y + range),
    });
    rect.set_min(geo::Coord {
        x: rect.min().x.min(center.x + range),
        y: rect.min().y.min(center.y + range),
    });
}

pub fn center_as_geographic_coordinates(rect: &BoundingBox) -> GeographicCoordinates {
    let center = rect.center();
    GeographicCoordinates {
        longitude: center.x,
        latitude: center.y,
    }
}

pub type BoundingBox = geo::Rect;

/*************************** /
#[derive(Debug, Clone, Copy)]
pub struct _BoundingBox {
    pub north: FGP,
    pub south: FGP,
    pub east: FGP,
    pub west: FGP,
}

impl fmt::Display for _BoundingBox {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{},{},{},{}",
            self.west, self.south, self.east, self.north
        )
        //write!(f, "I am A")
    }
}

impl Default for _BoundingBox {
    fn default() -> Self {
        Self::new()
    }
}

impl _BoundingBox {
    pub fn new() -> Self {
        _BoundingBox {
            north: FGP::MIN,
            south: FGP::MAX,
            east: FGP::MIN,
            west: FGP::MAX,
        }
    }

    pub const ZERO: Self = Self {
        north: 0.0,
        south: 0.0,
        east: 0.0,
        west: 0.0,
    };

    pub fn max_radius(&self) -> FGP {
        (self.east - self.west).max(self.north - self.south)
    }

    pub fn center_as_geographic_coordinates(&self) -> GeographicCoordinates {
        let latitude = (self.south + (self.north - self.south) / 2.) as f64;
        let longitude = (self.west + (self.east - self.west) / 2.) as f64;

        GeographicCoordinates {
            latitude,
            longitude,
        }
    }

    pub fn include(&mut self, position: &_GroundPosition) {
        self.north = self.north.max(position.y);
        self.south = self.south.min(position.y);
        self.east = self.east.max(position.x);
        self.west = self.west.min(position.x);
    }

    /*
     * Extend the area of the OSM object to the given range at last
     * @param {f32} range in meters - the minimum range of the bounding box
     */
    pub fn max_range(&mut self, range: FGP) {
        //println!("{self}");
        // range in meter to degres
        let range = range as FGP / LAT_FAKT as FGP;
        let center_north = (self.north - self.south) / 2. + self.south;
        let center_east = (self.east - self.west) / 2. + self.west;
        //println!("{range} {center_north} {center_east}");
        self.north = self.north.max(center_north + range);
        self.south = self.south.min(center_north - range);
        self.east = self.east.max(center_east + range);
        self.west = self.west.min(center_east - range);
        //println!("{self}");
    }

    pub fn shift(&mut self, shift: FGP) {
        self.north += shift;
        self.south += shift;
        self.east += shift;
        self.west += shift;
    }

    pub fn outside(&self, other: _BoundingBox) -> bool {
        self.east < other.west
            || self.west > other.east
            || self.north < other.south
            || self.south > other.north
    }
}
/ ***************************/

// A builiding without parts is its onw part or itselve is a part
#[derive(Clone, Debug)]
pub struct BuildingOrPart {
    pub id: u64,
    pub part: bool,
    pub footprint: Footprint,
    pub bounding_box_rotated: BoundingBox,
    // upper height of the wall, independend of / including the min_height
    pub wall_height: f64,
    pub min_height: f64,
    pub building_color: RenderColor,
    pub roof_shape: RoofShape,
    pub roof_height: f64,
    pub roof_angle: f64,
    pub roof_color: RenderColor,
}

pub type BuildingsAndParts = Vec<BuildingOrPart>;

#[derive(Deserialize, Debug, Clone)]
pub struct Member {
    #[serde(rename = "type")]
    pub member_type: String,
    #[serde(rename = "ref")]
    pub reference: u64,
    #[serde(rename = "role")]
    pub role: String,
}

pub type Members = Vec<Member>;
