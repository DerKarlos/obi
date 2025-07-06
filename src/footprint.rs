use std::ops::{Add, Sub};
extern crate earcutr; // not supported vor WASM?

use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::float::single::SingleFloatOverlay;
//e i_overlay::float::simplify::SimplifyShape;

use crate::kernel_in::{BoundingBox, GroundPosition, GroundPositions, Polygon, Polygons};

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
    _id: u64,
    // pub positions: GroundPositions,
    rotated_positions: GroundPositions,
    pub bounding_box: BoundingBox,
    pub shift: f32,
    pub center: GroundPosition,
    longest_distance: f32,
    pub longest_angle: f32,
    pub is_circular: bool,
    // is_clockwise: bool,
    pub polygons: Polygons,
    //pub holes: Vec<Footprint>,
}

impl Default for Footprint {
    fn default() -> Self {
        Self::new(4711)
    }
}

impl Footprint {
    pub fn new(_id: u64) -> Self {
        Self {
            _id,
            // positions: Vec::new(),
            rotated_positions: Vec::new(),
            bounding_box: BoundingBox::new(),
            shift: 0.0,
            center: GroundPosition::ZERO,
            longest_distance: 0.,
            longest_angle: 0.,
            is_circular: false,
            //is_clockwise: false,
            polygons: vec![vec![Vec::new()]], // first polygon still empty, for outer and some inner holes
                                              //holes: Vec::new(),
        }
    }

    pub fn first_polygon<'a>(&'a mut self) -> &'a mut Polygon {
        &mut self.polygons[0]
    }

    pub fn first_polygon_u<'a>(&'a self) -> &'a Polygon {
        &self.polygons[0]
    }

    pub fn first_outer<'a>(&'a mut self) -> &'a mut GroundPositions {
        &mut self.polygons[0][0]
    }

    pub fn first_outer_u<'a>(&'a self) -> &'a GroundPositions {
        &self.polygons[0][0]
    }

    pub fn push_position(&mut self, position: GroundPosition) {
        // self.positions.push(position);
        self.first_outer().push(position);
        self.bounding_box.include(&position);
        self.center.north += position.north;
        self.center.east += position.east;
    }

    pub fn close(&mut self) {
        // center
        let count = self.first_outer_u().len() as f32;
        self.center.north /= count;
        self.center.east /= count;

        let positions = &mut self.polygons[0][0];
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

    pub fn rotate(&mut self, roof_angle: f32) -> (BoundingBox, bool) {
        //println!("{len} rotate: {:?}", &self.polygons[0][0]);
        let mut bounding_box_rotated = BoundingBox::new();
        self.rotated_positions = Vec::new();
        for position in &self.polygons[0][0] {
            // Rotate against the actual angle to got 0 degrees
            let rotated_position = position.sub(self.center.clone()).rotate(-roof_angle);
            self.rotated_positions.push(rotated_position);
            bounding_box_rotated.include(&rotated_position);
        }

        let new_rotated_center_east = (bounding_box_rotated.east - bounding_box_rotated.west) / 2.0;
        let corretion_shift = new_rotated_center_east - bounding_box_rotated.east;
        bounding_box_rotated.shift(corretion_shift);
        self.shift = corretion_shift;
        for position in &mut self.rotated_positions {
            position.east += corretion_shift; // used in split_at_x_zero
        }

        let across = bounding_box_rotated.south - bounding_box_rotated.north
            > bounding_box_rotated.east - bounding_box_rotated.west;

        (bounding_box_rotated, across)
    }

    pub fn get_triangulate_indices(&self) -> Vec<usize> {
        //
        let mut vertices = Vec::<f32>::new();
        let mut holes_starts = Vec::<usize>::new();
        for position in self.first_outer_u() {
            // Hey earcut, why y before x ???
            vertices.push(position.north);
            vertices.push(position.east);
        }
        //println!("roof_po: {:?}", &vertices);

        for hole_index in 1..self.first_polygon_u().len() {
            let hole: &GroundPositions = &self.first_polygon_u()[hole_index];
            //  for hole in &self.first_polygon() { //holes {
            holes_starts.push(vertices.len() / 2);
            // println!("holes_starts: {:?}", &holes_starts);
            for position in hole {
                vertices.push(position.north);
                vertices.push(position.east);
            }
        }

        earcutr::earcut(&vertices, &holes_starts, 2).unwrap()
    }

    /// Splits the shape at x=0, returning two new shapes:
    /// - The first shape contains all parts with x ≤ 0
    /// - The second shape contains all parts with x ≥ 0
    /// - The last shape contains all parts of the outer
    pub fn split_at_x_zero(&mut self, angle: f32) -> (GroundPositions, GroundPositions) {
        let mut left_vertices = Vec::new();
        let mut right_vertices = Vec::new();
        let mut outer_vertices = Vec::new();

        let positions = &self.polygons[0][0];
        let n = self.rotated_positions.len();
        for i in 0..n {
            let current = self.rotated_positions[i];
            let next = self.rotated_positions[(i + 1) % n];
            outer_vertices.push(positions[i]);

            // If the current point is on the splitting line, add it to both shapes
            if current.east == 0.0 {
                left_vertices.push(positions[i]);
                right_vertices.push(positions[i]);
                println!(
                    "split split split split split split split split split split split split split split i:{i}"
                );
                continue;
            }

            // Add current point to appropriate side
            if current.east < 0.0 {
                left_vertices.push(positions[i]);
            } else {
                right_vertices.push(positions[i]);
            }

            //3 println!(" - Test1 i: {i} {current} {next}");
            // Check if the edge crosses the x=0 line      && true
            if current.east.signum() != next.east.signum() {
                // Calculate the intersection point
                let diagonally = -current.east / (next.east - current.east);
                let intersection_north = current.north + diagonally * (next.north - current.north);
                let intersection = GroundPosition {
                    north: intersection_north,
                    east: -self.shift,
                };

                // Add the intersection point to both shapes
                let intersection_rotated_back = intersection.rotate(angle).add(self.center);
                //3 println!(
                //3     "- Test2 i: {i} is_n: {intersection_north} {intersection} {intersection_rotated_back}"
                //3 );
                left_vertices.push(intersection_rotated_back);
                right_vertices.push(intersection_rotated_back);
                outer_vertices.push(intersection_rotated_back);
            }
        }

        self.polygons[0][0] = outer_vertices;
        (left_vertices, right_vertices)
    }

    // subttacting a hole of a polygon or a part inside a building
    pub fn subtract(&mut self, hole_positions: &Polygons) {
        const LOG: bool = false;
        // https://github.com/iShape-Rust/iOverlay/blob/main/readme/overlay_rules.md
        if LOG {
            println!(
                "{} ssss {} subj = {:?}",
                self._id,
                self.polygons.len(),
                &self.polygons
            );
            println!("cccc {}  clip = {:?}", hole_positions.len(), hole_positions);
        }

        let remaining = self
            .polygons //self.positions
            .overlay(hole_positions, OverlayRule::Difference, FillRule::EvenOdd);
        //  .                                              not working::Negative

        //println!(
        //    "subtract 1 {:?} ==== {:?} ---- {:?}",
        //    remaining, self.polygons, hole_positions
        //);

        // Reifenberg small remainings. Only ::Positive works. But will it hurt other models???
        // let remaining = remaining.simplify_shape(FillRule::Positive);
        // let remaining = remaining.simplify_shape_custom(FillRule::Positive);
        // simplify_shape_custom ??? https://docs.rs/i_overlay/latest/i_overlay/all.html   4.0.2
        //???println!("simplify_shape {:?}", remaining);

        if remaining.is_empty() {
            // todo: loop ways over parts. if way is gone, stop part loop
            println!("outer is gone {}", self._id);
            //println!(
            //    "subtract 2 {:?} ==== {:?} ---- {:?}",
            //    remaining, self.polygons, hole_positions
            //);

            self.polygons = remaining;
            return;
        }
        self.polygons = remaining;
        if self.first_polygon().is_empty() {
            println!("shape with no outer ...");
            return;
        }
        if LOG {
            println!(
                "Rrrrrrr [{}][{}][{}] = {:?}",
                self.polygons.len(),
                self.first_polygon().len(),
                self.polygons[0][0].len(),
                self.polygons
            );

            if self.polygons.len() > 1 || self.first_polygon().len() != 1 {
                if self.polygons.len() > 0 {
                    println!(
                        "shape subtract result.len()  [1][{}]",
                        self.polygons[0][0].len(),
                    );
                }
            }
        }

        return;
    }
}
