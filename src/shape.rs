use triangulation::{Delaunay, Point};
//e triangulate::{self, formats, Polygon};

use crate::kernel_in::{BoundingBox, GroundPosition};

#[derive(Clone, Debug)]
pub struct Shape {
    pub positions: Vec<GroundPosition>,
    pub bounding_box: BoundingBox,
    pub center: GroundPosition,
    longest_distance: f32,
    pub longest_angle: f32,
    //    is_clockwise: bool,
}

impl Shape {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            bounding_box: BoundingBox::new(),
            center: GroundPosition::NUL,
            longest_distance: 0.,
            longest_angle: 0.,
            //            is_clockwise: false,
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

    pub fn rotate(&self, roof_angle: f32) -> BoundingBox {
        let mut bounding_box_rotated = BoundingBox::new();
        for position in &self.positions {
            // why - negativ??? (see other lines)
            let rotated_position = position.rotate_around_center(-roof_angle, self.center);
            bounding_box_rotated.include(&rotated_position);
        }
        bounding_box_rotated
    }

    pub fn _get_triangulate_indices(&self) -> Vec<usize> {
        let mut roof_polygon: Vec<Point> = Vec::new();
        for position in &self.positions {
            let roof_point = Point::new(position.east, position.north);
            roof_polygon.push(roof_point);
        }

        let triangulation = Delaunay::new(&roof_polygon).unwrap();
        let indices = triangulation.dcel.vertices;
        //println!("triangles: {:?}",&indices);
        indices
    }
}
