// Internal Interface of the crate/lib between input modules/crates and a renderer

use crate::kernel_out::GpuPosition;

pub static LAT_FAKT: f64 = 111100.0; // 111285; // exactly enough  111120 = 1.852 * 1000.0 * 60  // 1 NM je Bogenminute: 1 Grad Lat = 60 NM = 111 km, 0.001 Grad = 111 m
pub static PI: f32 = std::f32::consts::PI;

use crate::shape::Shape;

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Debug)]
pub struct GroundPosition {
    pub north: f32,
    pub east: f32,
}

impl GroundPosition {
    pub const NUL: Self = Self {
        north: 0.,
        east: 0.,
    };

    pub fn to_gpu_position(&self, height: f32) -> GpuPosition {
        // Minus north because +north is -z in the GPU space.
        [self.east, height, -self.north]
    }

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

    pub fn rotate_around_center(self, angle: f32, center: GroundPosition) -> GroundPosition {
        //println!(
        //    "pre-rotate: angle {} position {:?}",
        //    angle.to_degrees(),
        //    //center,
        //    self
        //);
        let north = (self.north - center.north) * f32::cos(angle)
            - (self.east - center.east) * f32::sin(angle);
        let east = (self.east - center.east) * f32::cos(angle)
            - (self.north - center.north) * f32::sin(angle);
        // println!("rotated: north {} east {}", north, east);
        GroundPosition { north, east }
    }
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

    pub fn from_geo_range(geographic_coordinates: &GeographicCoordinates, range: f64) -> Self {
        let range = range / LAT_FAKT; // First test with 15 meter
        BoundingBox {
            north: (geographic_coordinates.latitude + range) as f32,
            south: (geographic_coordinates.latitude - range) as f32,
            west: (geographic_coordinates.longitude - range) as f32,
            east: (geographic_coordinates.longitude + range) as f32,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{},{},{},{}", self.west, self.south, self.east, self.north)
    }

    // let left_top = to_position(&CoordinatesAtGroundPositionNull, left, top);
    // println!("range: left_top={} url={}", left_top, url);
    // GET   /api/0.6/map?bbox=left,bottom,right,top

    pub fn include(&mut self, position: &GroundPosition) {
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
    pub _id: u64,
    pub _part: bool,
    pub footprint: Shape,
    //pub center: GroundPosition,
    //pub _bounding_box: BoundingBox,
    pub bounding_box_rotated: BoundingBox,
    // upper heit of the wall, independend of / including the min_height
    pub wall_height: f32,
    pub min_height: f32,
    pub color: RenderColor,
    pub roof_shape: RoofShape,
    pub roof_height: f32,
    pub roof_angle: f32,
    pub roof_color: RenderColor,
}
