// outer SHAPE of the building/part

// geo primitives
use geo::{
    Area, BooleanOps, BoundingRect, Distance, Euclidean, LineString, MultiPolygon, Point, Polygon,
    Rect, Rotate, Translate, TriangulateEarcut,
};

use geo::algorithm::unary_union;

use crate::kernel_in::{GroundPosition, GroundPositions};

static _O: usize = 0; // Just to silent lint, make some lines equal and to show, the Offset may also be 0

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
    rotated_positions: LineString,
    pub bounding_box: Rect,
    pub shift: f64,
    pub center: GroundPosition, // only use bb.center ???
    longest_distance: f64,
    pub longest_angle: f64,
    pub is_circular: bool,
    pub outer_one: GroundPositions,
    pub multipolygon: MultiPolygon,
}

impl Default for Footprint {
    fn default() -> Self {
        Self::new()
    }
}

impl Footprint {
    pub fn new() -> Self {
        Self {
            rotated_positions: LineString::new(Vec::new()),
            bounding_box: Rect::new(GroundPosition::zero(), GroundPosition::zero()),
            shift: 0.0,
            center: GroundPosition::zero(),
            longest_distance: 0.,
            longest_angle: 0.,
            is_circular: false,
            outer_one: Vec::new(),
            multipolygon: MultiPolygon::new(vec![Polygon::new(
                LineString::new(Vec::new()),
                Vec::new(),
            )]), // vec![vec![Vec::new()]], // first polygon still empty, for outer and some inner holes
        }
    }

    pub fn set_from_other(&mut self, other: &Footprint) {
        self.is_circular = other.is_circular;
        self.multipolygon = other.multipolygon.clone();
        //for pos in &self.polygons[0][0] {
        //    println!("(x: {},y: {}),", pos.east, pos.north);
        //}
        self.bounding_box = other.bounding_box;
        self.center = other.center;
        self.longest_angle = other.longest_angle;
        self.shift = other.shift;
    }

    pub fn push_position(&mut self, position: GroundPosition) {
        self.outer_one.push(position);
        // self.polygons[FIRST_POLYGON][OUTER_POLYGON].push(position);
        //self.bounding_box.
        //    .include(&position);
        //self.center.y += position.y;
        //self.center.x += position.x;
    }

    pub fn close(&mut self) {
        self.bounding_box = LineString::new(self.outer_one.clone())
            .bounding_rect()
            .unwrap();
        // center
        let count = self.outer_one.len(); // let count = self.polygons[FIRST_POLYGON][OUTER_POLYGON].len() as FGP;
        self.center = self.bounding_box.center();

        let positions = &mut self.outer_one; // &mut self.polygons[FIRST_POLYGON][OUTER_POLYGON];
        let mut clockwise_sum = 0.;
        let mut radius_max: f64 = 0.;
        let mut radius_min: f64 = 1.0e9;
        //let mut coords: Vec<Coord> = Vec::new();
        for (index, position) in positions.iter().enumerate() {
            //coords.push(position);
            let next = (index + 1) % positions.len();
            let next_position = positions[next];

            // angle
            let angle: f64 = f64::atan2(position.x - next_position.x, position.y - next_position.y);

            //let (distance, angle) = next_position.distance_angle_to_other(position);
            let distance = Euclidean.distance(next_position, *position);
            if self.longest_distance < distance {
                self.longest_distance = distance;
                self.longest_angle = angle;
            }
            // direction
            clockwise_sum += (next_position.y - position.y) * (next_position.x + position.x);
            // circular
            //let (distance, _) = self.center.distance_angle_to_other(position);
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
            (((radius_max - radius_min) / radius_max * 100.) as u32) < 10 && count >= 10;

        self.multipolygon = MultiPolygon::new(vec![Polygon::new(
            LineString::new(positions.clone()),
            Vec::new(),
        )]);
        self.outer_one = Vec::new();
    }

    pub fn rotate(&mut self, roof_angle: f64) -> Rect {
        // BoundingBox {
        let polygon = self.multipolygon.iter().next().unwrap();
        let linestring = polygon.exterior();
        self.rotated_positions =
            linestring.rotate_around_point(roof_angle.to_degrees(), self.center.into());

        // println!(            "{:?} rotated_positions: {:?}",            self.outer_one.len(),            self.rotated_positions        );
        let mut bounding_box_rotated = self.rotated_positions.bounding_rect().unwrap(); // BoundingBox::new();
        let new_rotated_center_y = bounding_box_rotated.height() / 2.;
        // (bounding_box_rotated.north - bounding_box_rotated.south) / 2.0;
        let corretion_shift = new_rotated_center_y - bounding_box_rotated.max().y; // .north;

        bounding_box_rotated = bounding_box_rotated.translate(0., corretion_shift); // bounding_box_rotated.shift(corretion_shift);
        self.rotated_positions = self.rotated_positions.translate(0., corretion_shift);
        self.shift = corretion_shift;

        bounding_box_rotated
    }

    pub fn get_area_size(&mut self) -> f64 {
        self.multipolygon.signed_area()
    }

    /***** This is just an ugly hack! i_overlay should be able to solve this - todo
    pub fn get_area_size(&mut self) -> FGP {
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
                let distance_a = FGP::sqrt(a * a + b * b);
                let a = n_1 - n_2;
                let b = e_1 - e_2;
                let distance_b = FGP::sqrt(a * a + b * b);
                let a = n_2 - n_0;
                let b = e_2 - e_0;
                let distance_c = FGP::sqrt(a * a + b * b);

                let a = distance_a;
                let b = distance_b;
                let c = distance_c;
                area_spliter_size += 0.25
                    * FGP::sqrt(FGP::abs(
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
    ****/
    pub fn get_triangulates(&self, polygon_index: usize) -> Vec<usize> {
        //
        for (index, polygon) in self.multipolygon.iter().enumerate() {
            if index == polygon_index {
                return polygon.earcut_triangles_raw().triangle_indices;
            };
        }
        Vec::new()
    }

    /// Splits the shape at x=0, returning two new shapes:
    /// - The first shape contains all parts with x ≤ 0
    /// - The second shape contains all parts with x ≥ 0
    /// - The last shape contains all parts of the outer
    pub fn split_at_y_zero(&mut self, _angle: f32) -> (MultiPolygon, MultiPolygon) {
        //
        let rect_up = Rect::new(
            Point::new(1e9, 1e9), // max x y
            Point::new(-1e9, 0.), // min x y
        )
        .to_polygon();
        let rect_low = Rect::new(
            Point::new(1e9, 0.),    // max x y
            Point::new(-1e9, -1e9), // min x y
        )
        .to_polygon();

        // let polygon = self.polygons.iter().next().unwrap();

        let up = self.multipolygon.difference(&rect_up);
        let low = self.multipolygon.difference(&rect_low);

        let polygons = vec![up.clone(), low.clone()];
        let outer = unary_union(&polygons);

        /*
        let mut low_vertices = Vec::new();
        let mut up_vertices = Vec::new();
        let mut outer_vertices = Vec::new();

        //let positions = &self.polygons[FIRST_POLYGON][OUTER_POLYGON];
        let n = self.rotated_positions.points().len();
        for (i, current) in self.rotated_positions.points().enumerate().take(n) {
            let next = self.rotated_positions[(i + 1) % n];
            let points = positions.points();
            let p = points. ();
            outer_vertices.push(point);

            // If the current point is on the splitting line, add it to both shapes
            if current.y() == 0.0 {
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

        self.polygons[FIRST_POLYGON][OUTER_POLYGON] = outer_vertices;
        (low_vertices, up_vertices)
        */
        self.multipolygon = outer;
        (low, up)
    }

    // subttacting a hole of a polygon or a part inside a building - todo use the one line "inline"
    pub fn subtract(&mut self, other_a_hole: &Footprint) {
        // println!("### other_a_hole othr: {:?}", other_a_hole);
        // println!("### subtract othr: {:?}", othr);
        // let remaining = self.multipolygon.difference(&other_a_hole.multipolygon);
        self.multipolygon = self.multipolygon.difference(&other_a_hole.multipolygon);

        // let sa = self.multipolygon.signed_area();
        // let oa = other_a_hole.multipolygon.signed_area();
        // let ra = remaining.signed_area();
        // let x0 = sa - oa - ra;
        // if x0 > 0.1 {
        //     println!("ta: {sa} oa: {oa} ra: {ra} 0: {:?}", x0);
        // }
    }

    /*******
    fn line_string_to_positions(&self, line_string: LineString) -> GroundPositions {
        let mut positions: Vec<GroundPosition> = Vec::new();
        for point in line_string {
            positions.push(GroundPosition {
                north: point.y as FGP,
                east: point.x as FGP,
            })
        }
        positions
    }

    fn polygons_from_geo(&mut self, multi_polygon: MultiPolygon) -> Polygons {
        let mut polygons = Polygons::new();
        for geo_polygon in multi_polygon {
            let mut polygon: crate::kernel_in::Polygon = vec![];
            let (outer, holes) = geo_polygon.into_inner();
            polygon.push(self.line_string_to_positions(outer));
            for hole in holes {
                polygon.push(self.line_string_to_positions(hole));
            }
            polygons.push(polygon);
        }
        polygons
    }

    fn to_geo_line_string(&self, polygon_index: usize, line_string_index: usize) -> LineString {
        let mut coords = vec![];
        for position in &self.polygons[polygon_index][line_string_index] {
            coords.push(Coord {
                x: position.east as f64,
                y: position.north as f64,
            });
        }
        LineString::new(coords)
    }

    // todo: that init is a hack!!!
    fn to_geo_polygon(&self, polygon_index: usize) -> Polygon {
        //println!("### self.pol* {:?}", &self.polygons);
        let mut interiors: Vec<LineString> = Vec::new();
        let polygon = &self.polygons[polygon_index];

        for (line_string_index, _positions) in polygon.iter().enumerate().skip(FIRST_HOLE_INDEX) {
            interiors.push(self.to_geo_line_string(polygon_index, line_string_index));
        }
        //println!(
        //    "### polygon {polygon_index} [OUTER_POLYGON] {:?}",
        //    polygon[OUTER_POLYGON]
        //);
        let exteriors = self.to_geo_line_string(polygon_index, OUTER_POLYGON);
        //println!("### exteriors {:?}", exteriors);

        Polygon::new(exteriors, interiors)
    }

    fn to_geo_multi_polygon(&self) -> MultiPolygon {
        let mut poligons = vec![];
        for (i, _polygon) in self.polygons.iter().enumerate() {
            poligons.push(self.to_geo_polygon(i));
        }

        MultiPolygon::new(poligons)
    }

    *******/

    pub fn other_is_inside(&self, other: &Footprint) -> bool {
        //let other_polygon = other.to_geo_polygon(FIRST_POLYGON);
        //let self_polygon = self.to_geo_polygon(FIRST_POLYGON);

        //let remaining_other_polygon = self_polygon.difference(&other_polygon);
        // The regions of `self` which are not in `other`.
        let remaining_other_polygon = other.multipolygon.difference(&self.multipolygon);

        //let self_exterior = self_polygon.exterior();
        //let other_exterior = other_polygon.exterior();
        //let remaining_other_polygon = other_exterior.difference(self_exterior);
        let other_area = other.multipolygon.unsigned_area();
        let remaining_other_area = remaining_other_polygon.unsigned_area();
        let remaining = remaining_other_area / other_area;
        #[cfg(debug_assertions)]
        {
            if remaining > 0.01 && remaining < 0.999 {
                println!("remaining: {remaining} {remaining_other_area}/{other_area}");
            }
        }
        remaining < 0.01
    }
}
