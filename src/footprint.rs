use std::ops::{Add, Sub};
extern crate earcutr; // not supported vor WASM?

use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::float::single::SingleFloatOverlay;

use crate::kernel_in::{BoundingBox, GroundPosition};

#[derive(Clone, Debug)]
pub struct Footprint {
    _id: u64,
    pub positions: Vec<GroundPosition>,
    rotated_positions: Vec<GroundPosition>,
    pub bounding_box: BoundingBox,
    pub shift: f32,
    pub center: GroundPosition,
    longest_distance: f32,
    pub longest_angle: f32,
    pub is_circular: bool,
    // is_clockwise: bool,
    pub multipollygon: Vec<Vec<Vec<GroundPosition>>>,
    pub holes: Vec<Footprint>,
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
            positions: Vec::new(),
            rotated_positions: Vec::new(),
            bounding_box: BoundingBox::new(),
            shift: 0.0,
            center: GroundPosition::ZERO,
            longest_distance: 0.,
            longest_angle: 0.,
            is_circular: false,
            //is_clockwise: false,
            multipollygon: vec![Vec::new()], // first polygon still empty, for outer and some inner holes
            holes: Vec::new(),
        }
    }

    pub fn push_position(&mut self, position: GroundPosition) {
        self.positions.push(position);
        self.bounding_box.include(&position);
        self.center.north += position.north;
        self.center.east += position.east;
    }

    pub fn push_hole(&mut self, mut hole: Footprint) {
        if self.multipollygon[0].is_empty() {
            println!("??? push hole to mepty1: {}", self._id);
        }

        if self.positions.is_empty() {
            println!("??? push hole to mepty2: {}", self._id);
        }

        hole.positions.reverse();
        self.multipollygon[0].push(hole.positions.clone());
        self.holes.push(hole);
    }

    pub fn close(&mut self) {
        // center
        let count = self.positions.len() as f32;
        self.center.north /= count;
        self.center.east /= count;

        let mut clockwise_sum = 0.;
        let mut radius_max: f32 = 0.;
        let mut radius_min: f32 = 1.0e9;
        for (index, position) in self.positions.iter().enumerate() {
            let next = (index + 1) % self.positions.len();
            let next_position = self.positions[next];

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
            self.positions.reverse();
        }
        // radius iregularity is less but x% of the radius
        self.is_circular =
            (((radius_max - radius_min) / radius_max * 100.) as u32) < 10 && count >= 10.;
        //println!(
        //    "r max/min {radius_max}/{radius_min} len: {count} = {:?}",
        //    (radius_max - radius_min) / radius_max * 100.
        //);
        self.multipollygon[0].push(self.positions.clone());
    }

    // Shape.rotate
    pub fn rotate(&mut self, roof_angle: f32) -> BoundingBox {
        let mut bounding_box_rotated = BoundingBox::new();
        for position in &self.positions {
            // Rotate against the actual angle to got 0 degrees
            let rotated_position = position.sub(self.center).rotate(-roof_angle);
            self.rotated_positions.push(rotated_position);
            bounding_box_rotated.include(&rotated_position);
        }

        //3 println!(
        //3     "e: {:?} w{:?}",
        //3     bounding_box_rotated.east, bounding_box_rotated.west
        //3 );
        let new_rotated_center_east = (bounding_box_rotated.east - bounding_box_rotated.west) / 2.0;
        //                                      8   -                       -4 = 12 / 2 = 6
        let corretion_shift = new_rotated_center_east - bounding_box_rotated.east;
        //                         6     -            8   = -2
        bounding_box_rotated.shift(corretion_shift);
        self.shift = corretion_shift;
        for position in &mut self.rotated_positions {
            //3 println!(
            //3     "rot east: {:?}+{:?}={:?}",
            //3     position.east.clone(),
            //3     corretion_shift,
            //3     position.east + corretion_shift
            //3 );
            position.east += corretion_shift; // used in split_at_x_zero
        }

        bounding_box_rotated
    }

    pub fn get_triangulate_indices(&self) -> Vec<usize> {
        //
        let mut vertices = Vec::<f32>::new();
        let mut holes_starts = Vec::<usize>::new();
        for position in &self.positions {
            // Hey earcut, why y before x ???
            vertices.push(position.north);
            vertices.push(position.east);
        }
        //println!("roof_po: {:?}", &vertices);

        for hole in &self.holes {
            holes_starts.push(vertices.len() / 2);
            // println!("holes_starts: {:?}", &holes_starts);
            for position in &hole.positions {
                vertices.push(position.north);
                vertices.push(position.east);
            }
        }

        //if !holes_starts.is_empty() {
        //    if holes_starts[0] == 0 {
        //        println!(
        //            "{} bad holes_starts: {:?} vertices: {:?} ",
        //            self._id, &holes_starts, &vertices
        //        );
        //        holes_starts = Vec::new();
        //    }
        //}
        earcutr::earcut(&vertices, &holes_starts, 2).unwrap()
    }

    /// Splits the shape at x=0, returning two new shapes:
    /// - The first shape contains all parts with x ≤ 0
    /// - The second shape contains all parts with x ≥ 0
    /// - The last shape contains all parts of the outer
    pub fn split_at_x_zero(&mut self, angle: f32) -> (Vec<GroundPosition>, Vec<GroundPosition>) {
        let mut left_vertices = Vec::new();
        let mut right_vertices = Vec::new();
        let mut outer_vertices = Vec::new();

        let n = self.rotated_positions.len();
        for i in 0..n {
            let current = self.rotated_positions[i];
            let next = self.rotated_positions[(i + 1) % n];
            //outer_vertices.push(current); //ttt
            outer_vertices.push(self.positions[i]);

            // If the current point is on the splitting line, add it to both shapes
            if current.east == 0.0 {
                left_vertices.push(self.positions[i]);
                right_vertices.push(self.positions[i]);
                println!(
                    "split split split split split split split split split split split split split split i:{i}"
                );
                continue;
            }

            // Add current point to appropriate side
            if current.east < 0.0 {
                left_vertices.push(self.positions[i]);
            } else {
                right_vertices.push(self.positions[i]);
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

        self.positions = outer_vertices;
        (left_vertices, right_vertices)
    }

    pub fn substract(&mut self, other_positions: &Vec<GroundPosition>) -> bool {
        const LOG: bool = false;
        // https://github.com/iShape-Rust/iOverlay/blob/main/readme/overlay_rules.md
        if LOG {
            println!(
                "{} ssss {} subj = {:?}",
                self._id,
                self.positions.len(),
                &self.positions
            );
            println!(
                "cccc {}  clip = {:?}",
                other_positions.len(),
                other_positions
            );
        }

        let substraction = self
            .multipollygon //self.positions
            .overlay(other_positions, OverlayRule::Difference, FillRule::Negative);

        //println!(
        //    "äää {:?} ==== {:?} ---- {:?}",
        //    substraction, self.multipollygon, other_positions
        //);

        if substraction.is_empty() {
            // outer is gone, remove way
            self.positions = Vec::new();
            //println!("outer is gone, remove way {}", self._id);
            //println!(
            //    "äää {:?} ==== {:?} ---- {:?}",
            //    substraction, self.multipollygon, other_positions
            //);
            self.multipollygon = substraction;
            return false;
        }
        self.multipollygon = substraction;
        if self.multipollygon[0].is_empty() {
            println!("shape with no outer ...");
            return false;
        }
        if LOG {
            println!(
                "Rrrrrrr [{}][{}][{}] = {:?}",
                self.multipollygon.len(),
                self.multipollygon[0].len(),
                self.multipollygon[0][0].len(),
                self.multipollygon
            );

            if self.multipollygon.len() > 1 || self.multipollygon[0].len() != 1 {
                if self.multipollygon.len() > 0 {
                    println!(
                        "shape substract result.len()  [1][{}]",
                        self.multipollygon[0][0].len(),
                    );
                }
            }
        }

        // todo: prozess holes in building
        // todo: process all remainings of the cutted building
        // seach vor the lagest remaining part and use it below. Better use all???
        let mut max: usize = 0;
        let mut ind: usize = 0;
        for (i, remaining) in self.multipollygon.iter().enumerate() {
            if remaining[0].len() > max {
                max = remaining[0].len();
                ind = i;
            }
        }

        // 0o = first remaining shape, 0o = outer before the holes
        let resulti0 = self.multipollygon[ind][0].clone();
        // println!(
        //     "{} shape multipollygon {}!={}",
        //     self._id,
        //     self.multipollygon[ind][0].len(),
        //     self.positions.len()
        // );
        let changed = self.multipollygon[ind][0].len() != self.positions.len();
        self.positions = resulti0;
        changed
    }
}
