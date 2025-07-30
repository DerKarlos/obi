// outer SHAPE of the building/part

use std::ops::{Add, Sub};
extern crate earcutr; // not supported vor WASM?

use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::float::single::SingleFloatOverlay;

use crate::kernel_in::{
    BoundingBox, FIRST_HOLE_INDEX, FIRST_POLYGON, GroundPosition, GroundPositions, POLYGON_OUTER,
    Polygons,
};

static O: usize = 0; // Just to silent lint, make some lines equal and to show, the Offset may also be 0

pub enum Orientation {
    None,
    Along,
    Across,
    ByLongestSide,
    ByAngleValue,
    ByNauticDirction,
}

#[derive(Clone, Debug)]
pub struct Footprint {
    rotated_positions: GroundPositions,
    pub bounding_box: BoundingBox,
    pub shift: f32,
    pub center: GroundPosition,
    longest_distance: f32,
    pub longest_angle: f32,
    pub is_circular: bool,
    pub polygons: Polygons,
}

impl Default for Footprint {
    fn default() -> Self {
        Self::new()
    }
}

impl Footprint {
    pub fn new() -> Self {
        Self {
            rotated_positions: Vec::new(),
            bounding_box: BoundingBox::new(),
            shift: 0.0,
            center: GroundPosition::ZERO,
            longest_distance: 0.,
            longest_angle: 0.,
            is_circular: false,
            polygons: vec![vec![Vec::new()]], // first polygon still empty, for outer and some inner holes
                                              //holes: Vec::new(),
        }
    }

    pub fn set(&mut self, other: &Footprint) {
        self.is_circular = other.is_circular;
        self.polygons = other.polygons.clone();
        self.bounding_box = other.bounding_box;
        self.center = other.center;
        self.longest_angle = other.longest_angle;
        self.shift = other.shift;
    }

    pub fn push_position(&mut self, position: GroundPosition) {
        self.polygons[FIRST_POLYGON][POLYGON_OUTER].push(position);
        self.bounding_box.include(&position);
        self.center.north += position.north;
        self.center.east += position.east;
    }

    pub fn close(&mut self) {
        // center
        let count = self.polygons[FIRST_POLYGON][POLYGON_OUTER].len() as f32;
        self.center.north /= count;
        self.center.east /= count;

        let positions = &mut self.polygons[FIRST_POLYGON][POLYGON_OUTER];
        let mut clockwise_sum = 0.;
        let mut radius_max: f32 = 0.;
        let mut radius_min: f32 = 1.0e9;
        for (index, position) in positions.iter().enumerate() {
            let next = (index + 1) % positions.len();
            let next_position = positions[next];

            // angle
            let (distance, angle) = next_position.distance_angle_to_other(position);
            if self.longest_distance < distance {
                self.longest_distance = distance;
                self.longest_angle = angle;
            }
            // direction
            clockwise_sum +=
                (next_position.north - position.north) * (next_position.east + position.east);
            // circular
            let (distance, _) = self.center.distance_angle_to_other(position);
            radius_max = radius_max.max(distance);
            radius_min = radius_min.min(distance);
        }
        // https://stackoverflow.com/questions/1165647/how-to-determine-if-a-list-of-polygon-points-are-in-clockwise-order
        let is_clockwise = clockwise_sum > 0.0;
        if !is_clockwise {
            positions.reverse();
        }
        // radius iregularity is less but x% of the radius
        self.is_circular =
            (((radius_max - radius_min) / radius_max * 100.) as u32) < 10 && count >= 10.;
    }

    pub fn rotate(&mut self, roof_angle: f32) -> BoundingBox {
        //println!("{len} rotate: {:?}", &self.polygons[FIRST_POLYGON][POLYGON_OUTER]);
        let mut bounding_box_rotated = BoundingBox::new();
        self.rotated_positions = Vec::new();
        for position in &self.polygons[FIRST_POLYGON][POLYGON_OUTER] {
            // Rotate against the actual angle to got 0 degrees
            let rotated_position = position.sub(self.center.clone()).rotate(-roof_angle);
            self.rotated_positions.push(rotated_position);
            bounding_box_rotated.include(&rotated_position);
        }

        let new_rotated_center_north =
            (bounding_box_rotated.north - bounding_box_rotated.south) / 2.0;
        let corretion_shift = new_rotated_center_north - bounding_box_rotated.north;
        bounding_box_rotated.shift(corretion_shift);
        self.shift = corretion_shift;
        for position in &mut self.rotated_positions {
            position.north += corretion_shift; // used in split_at_y_zero
        }

        bounding_box_rotated
    }

    // This is just an ugly hack! i_overlay should be able to solve this - todo
    pub fn get_area_size(&mut self) -> f32 {
        let mut area_size = 0.0;
        let mut index = 0;
        while index < self.polygons.len() {
            //for (index, _polygon) in self.polygons.iter().enumerate() {
            let (indices, vertices) = self.get_triangulate_indices(index);
            let mut area_spliter_size = 0.0;
            for index_to_indices in 0..indices.len() / 3 {
                let vertice_index_0 = indices[index_to_indices * 3 + O];
                let vertice_index_1 = indices[index_to_indices * 3 + 1];
                let vertice_index_2 = indices[index_to_indices * 3 + 2];

                let n_0 = vertices[vertice_index_0 * 2 + O];
                let e_0 = vertices[vertice_index_0 * 2 + 1];
                let n_1 = vertices[vertice_index_1 * 2 + O];
                let e_1 = vertices[vertice_index_1 * 2 + 1];
                let n_2 = vertices[vertice_index_2 * 2 + O];
                let e_2 = vertices[vertice_index_2 * 2 + 1];

                let a = n_0 - n_1;
                let b = e_0 - e_1;
                let distance_a = f32::sqrt(a * a + b * b);
                let a = n_1 - n_2;
                let b = e_1 - e_2;
                let distance_b = f32::sqrt(a * a + b * b);
                let a = n_2 - n_0;
                let b = e_2 - e_0;
                let distance_c = f32::sqrt(a * a + b * b);

                let a = distance_a;
                let b = distance_b;
                let c = distance_c;
                area_spliter_size += 0.25
                    * f32::sqrt(f32::abs(
                        (a + b + c) * (-a + b + c) * (a - b + c) * (a + b - c),
                    ));
                // println!("    area: {area}");
            }
            // compendate error of subtract-crate
            if area_spliter_size < 0.1 && self.polygons.len() > 1 {
                self.polygons.remove(index);
            } else {
                area_size += area_spliter_size;
                index += 1;
            }
        }

        area_size
    }

    pub fn get_triangulate_indices(&self, polygon_index: usize) -> (Vec<usize>, Vec<f32>) {
        //

        let mut vertices = Vec::<f32>::new();
        let mut holes_starts = Vec::<usize>::new();

        for position in &self.polygons[polygon_index][POLYGON_OUTER] {
            // Hey earcut, why y before x ???
            vertices.push(position.north);
            vertices.push(position.east);
        }
        //println!("roof_po: {:?}", &vertices);

        for hole_index in FIRST_HOLE_INDEX..self.polygons[polygon_index].len() {
            let hole: &GroundPositions = &self.polygons[polygon_index][hole_index];
            holes_starts.push(vertices.len() / 2);
            // println!("holes_starts: {:?}", &holes_starts);
            for position in hole {
                vertices.push(position.north);
                vertices.push(position.east);
            }
        }

        let indices = earcutr::earcut(&vertices, &holes_starts, 2).unwrap();

        (indices, vertices)
    }

    /// Splits the shape at x=0, returning two new shapes:
    /// - The first shape contains all parts with x ≤ 0
    /// - The second shape contains all parts with x ≥ 0
    /// - The last shape contains all parts of the outer
    pub fn split_at_y_zero(&mut self, angle: f32) -> (GroundPositions, GroundPositions) {
        let mut low_vertices = Vec::new();
        let mut up_vertices = Vec::new();
        let mut outer_vertices = Vec::new();

        let positions = &self.polygons[FIRST_POLYGON][POLYGON_OUTER];
        let n = self.rotated_positions.len();
        for i in 0..n {
            let current = self.rotated_positions[i];
            let next = self.rotated_positions[(i + 1) % n];
            outer_vertices.push(positions[i]);

            // If the current point is on the splitting line, add it to both shapes
            if current.north == 0.0 {
                low_vertices.push(positions[i]);
                up_vertices.push(positions[i]);
                println!(
                    "split split split split split split split split split split split split split split i:{i}"
                );
                continue;
            }

            // Add current point to appropriate side
            if current.north < 0.0 {
                low_vertices.push(positions[i]);
            } else {
                up_vertices.push(positions[i]);
            }

            //3 println!(" - Test1 i: {i} {current} {next}");
            // Check if the edge crosses the x=0 line      && true
            if current.north.signum() != next.north.signum() {
                // Calculate the intersection point
                let diagonally = -current.north / (next.north - current.north);
                let intersection_north = current.east + diagonally * (next.east - current.east);
                let intersection = GroundPosition {
                    east: intersection_north,
                    north: -self.shift,
                };

                // Add the intersection point to both shapes
                let intersection_rotated_back = intersection.rotate(angle).add(self.center);
                low_vertices.push(intersection_rotated_back);
                up_vertices.push(intersection_rotated_back);
                outer_vertices.push(intersection_rotated_back);
            }
        }

        self.polygons[FIRST_POLYGON][POLYGON_OUTER] = outer_vertices;
        (low_vertices, up_vertices)
    }

    // subttacting a hole of a polygon or a part inside a building
    pub fn subtract(&mut self, hole_positions: &Polygons) {
        let remaining =
            self.polygons
                .overlay(hole_positions, OverlayRule::Difference, FillRule::Positive);
        //  .                                                  not working::Negative

        // simplify did not realy work, just cut it always away
        // simplify_shape_custom ??? https://docs.rs/i_overlay/latest/i_overlay/all.html   4.0.2

        if remaining.is_empty() {
            println!("outer is gone");
            self.polygons = remaining;
            return;
        }
        self.polygons = remaining;
        if self.polygons[FIRST_POLYGON].is_empty() {
            println!("shape with no outer ...");
        }
    }
}
