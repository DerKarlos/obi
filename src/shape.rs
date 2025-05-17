use std::ops::Add;
// use triangulation::{Delaunay, Point};
//
// use triangulate::formats;
// use triangulate::Polygon;
// use triangulate::{ListFormat, PolygonList};
extern crate earcutr;

use crate::kernel_in::{BoundingBox, GroundPosition};

#[derive(Clone, Debug)]
pub struct Shape {
    pub positions: Vec<GroundPosition>,
    rotated_positions: Vec<GroundPosition>,
    pub bounding_box: BoundingBox,
    pub center: GroundPosition,
    longest_distance: f32,
    pub longest_angle: f32,
    // is_clockwise: bool,
}

impl Shape {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            rotated_positions: Vec::new(),
            bounding_box: BoundingBox::new(),
            center: GroundPosition::ZERO,
            longest_distance: 0.,
            longest_angle: 0.,
            //is_clockwise: false,
        }
    }

    pub fn push(&mut self, position: GroundPosition) {
        self.positions.push(position);
        self.bounding_box.include(&position);
        self.center.north += position.north;
        self.center.east += position.east;
    }

    pub fn close(&mut self) {
        // center
        let count = self.positions.len() as f32;
        self.center.north /= count;
        self.center.east /= count;

        let mut clockwise_sum = 0.;
        for (index, position) in self.positions.iter().enumerate() {
            let next = (index + 1) % self.positions.len();
            let next_position = self.positions[next];

            // angle
            let (distance, angle) = next_position.distance_angle_to_other(&position);
            if self.longest_distance < distance {
                self.longest_distance = distance;
                self.longest_angle = angle;
            }
            // direction
            clockwise_sum +=
                (next_position.north - position.north) * (next_position.east + position.east);
        }
        // https://stackoverflow.com/questions/1165647/how-to-determine-if-a-list-of-polygon-points-are-in-clockwise-order
        let is_clockwise = clockwise_sum > 0.0;
        if !is_clockwise {
            self.positions.reverse();
        }
    }

    pub fn rotate(&mut self, roof_angle: f32) -> BoundingBox {
        let mut bounding_box_rotated = BoundingBox::new();
        for position in &self.positions {
            // why - negativ??? (see other lines)
            let rotated_position = position.rotate_around_center(-roof_angle, self.center);
            self.rotated_positions.push(rotated_position);
            bounding_box_rotated.include(&rotated_position);
        }
        bounding_box_rotated
    }

    pub fn get_triangulate_indices(&self) -> Vec<usize> {
        //
        let mut vertices = Vec::<f32>::new();
        for position in &self.positions {
            // why y before x ???
            vertices.push(position.north);
            vertices.push(position.east);
        }
        //println!("roof_po: {:?}", &vertices);

        let indices = earcutr::earcut(&vertices, &[], 2).unwrap();
        //println!("{:?}", indices);

        indices
    }

    /// Splits the shape at x=0, returning two new shapes:
    /// - The first shape contains all parts with x ≤ 0
    /// - The second shape contains all parts with x ≥ 0
    /// - The last shape contains all parts of the outer
    pub fn split_at_x_zero(&mut self, angle: f32) -> (Vec<GroundPosition>, Vec<GroundPosition>) {
        let mut left_vertices = Vec::new();
        let mut right_vertices = Vec::new();
        let mut outer_vertices = Vec::new();

        //self.rotated_positions.remove(0); //ttt
        let n = self.rotated_positions.len();
        for i in 0..n {
            let current = self.rotated_positions[i];
            let next = self.rotated_positions[(i + 1) % n];
            outer_vertices.push(self.positions[i]);

            // If the current point is on the splitting line, add it to both shapes
            if current.east == 0.0 {
                left_vertices.push(self.positions[i]);
                right_vertices.push(self.positions[i]);
                println!("split i:{i}");
                continue;
            }

            // Add current point to appropriate side
            if current.east < 0.0 {
                left_vertices.push(self.positions[i]);
            } else {
                right_vertices.push(self.positions[i]);
            }

            println!("ttt1 i: {i} {current} {next}");
            // Check if the edge crosses the x=0 line
            if current.east.signum() != next.east.signum() {
                // Calculate the intersection point
                let diagonally = -current.east / (next.east - current.east);
                let intersection_north = current.north + diagonally * (next.north - current.north);
                let intersection = GroundPosition {
                    north: intersection_north,
                    east: 0.0,
                };

                println!("ttt2 i: {i} is_n: {intersection_north}");
                // Add the intersection point to both shapes
                let intersection_rotated_back = intersection.rotate(angle).add(self.center);
                println!("ttt3 i: {i} is_n: {intersection_north} {intersection} {intersection_rotated_back}");
                left_vertices.push(intersection_rotated_back);
                right_vertices.push(intersection_rotated_back);
                outer_vertices.push(intersection_rotated_back);
            }
        }

        self.positions = outer_vertices;
        (left_vertices, right_vertices)
    }
}
