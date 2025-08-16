// outer SHAPE of the building/part

use std::ops::{Add, Sub};
extern crate earcutr; // not supported vor WASM?

//use i_overlay::core::fill_rule::FillRule;
//use i_overlay::core::overlay_rule::OverlayRule;
//use i_overlay::float::single::SingleFloatOverlay;

// geo primitives
use geo::{BooleanOps, Contains, Coord, CoordsIter, LineString, MultiPolygon, Polygon};

use crate::kernel_in::{
    BoundingBox, FIRST_HOLE_INDEX, FIRST_POLYGON, GroundPosition, GroundPositions, OUTER_POLYGON,
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
    pub pol_init: Polygons,
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
            pol_init: vec![vec![Vec::new()]],
        }
    }

    pub fn set(&mut self, other: &Footprint) {
        self.is_circular = other.is_circular;
        self.polygons = other.polygons.clone();
        //for pos in &self.polygons[0][0] {
        //    println!("(x: {},y: {}),", pos.east, pos.north);
        //}
        self.bounding_box = other.bounding_box;
        self.center = other.center;
        self.longest_angle = other.longest_angle;
        self.shift = other.shift;
    }

    pub fn push_position(&mut self, position: GroundPosition) {
        self.polygons[FIRST_POLYGON][OUTER_POLYGON].push(position);
        self.bounding_box.include(&position);
        self.center.north += position.north;
        self.center.east += position.east;
    }

    pub fn close(&mut self) {
        // center
        let count = self.polygons[FIRST_POLYGON][OUTER_POLYGON].len() as f32;
        self.center.north /= count;
        self.center.east /= count;

        let positions = &mut self.polygons[FIRST_POLYGON][OUTER_POLYGON];
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
        for position in &self.polygons[FIRST_POLYGON][OUTER_POLYGON] {
            // Rotate against the actual angle to got 0 degrees
            let rotated_position = position.sub(self.center).rotate(-roof_angle);
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

        for position in &self.polygons[polygon_index][OUTER_POLYGON] {
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

        let positions = &self.polygons[FIRST_POLYGON][OUTER_POLYGON];
        let n = self.rotated_positions.len();
        for (i, current) in self.rotated_positions.iter().enumerate().take(n) {
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

        self.polygons[FIRST_POLYGON][OUTER_POLYGON] = outer_vertices;
        (low_vertices, up_vertices)
    }

    // subttacting a hole of a polygon or a part inside a building
    pub fn subtract(&mut self, other_a_hole: &Footprint) {
        let this = self.to_geo_multi_polygon();
        let othr = other_a_hole.to_geo_multi_polygon();
        let rema = this.difference(&othr);
        //let ta = this.signed_area();
        //let oa = othr.signed_area();
        //let ra = rema.signed_area();
        //println!("ta: {ta} oa: {oa} ra: {ra} 0: {:?}", ta - oa - ra);
        let remaining = self.from_geo(rema);

        //let remaining =
        //    self.polygons
        //        .overlay(hole_positions, OverlayRule::Difference, FillRule::Positive);
        ////  .                                                  not working::Negative

        // simplify did not realy work, just cut it always away
        // simplify_shape_custom ??? https://docs.rs/i_overlay/latest/i_overlay/all.html   4.0.2

        if remaining.is_empty() {
            #[cfg(debug_assertions)]
            println!("outer is gone");
            self.polygons = remaining;
            return;
        }
        self.polygons = remaining;
        if self.polygons[FIRST_POLYGON].is_empty() {
            println!("shape with no outer ...");
        }
    }

    //ääää
    fn line_string_to_positions(&self, line_string: LineString) -> GroundPositions {
        let mut positions: Vec<GroundPosition> = Vec::new();
        for point in line_string {
            positions.push(GroundPosition {
                north: point.y as f32,
                east: point.x as f32,
            })
        }
        positions
    }

    fn from_geo(&mut self, multi_polygon: MultiPolygon) -> Polygons {
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
    fn to_geo_polygon(&self, polygon_index: usize, init: bool) -> Polygon {
        let mut interiors: Vec<LineString> = Vec::new();
        let mut polygon = &self.polygons[polygon_index];

        if init {
            if !self.pol_init[0][0].is_empty() {
                polygon = &self.pol_init[polygon_index];
                println!("pol_init");
            }
        }

        for (line_string_index, _positions) in polygon.iter().enumerate().skip(FIRST_HOLE_INDEX) {
            interiors.push(self.to_geo_line_string(polygon_index, line_string_index));
        }
        println!("polygon[0][0] {:?}", polygon[FIRST_POLYGON][OUTER_POLYGON]);
        let exteriors = self.to_geo_line_string(polygon_index, OUTER_POLYGON);
        println!("exteriors {:?}", exteriors[0]);

        Polygon::new(exteriors, interiors)
    }

    fn to_geo_multi_polygon(&self) -> MultiPolygon {
        let mut poligons = vec![];
        for (i, _polygon) in self.polygons.iter().enumerate() {
            poligons.push(self.to_geo_polygon(i, false));
        }
        MultiPolygon::new(poligons)
    }

    pub fn other_is_inside(&self, other: &Footprint) -> bool {
        let other_polygon = other.to_geo_polygon(FIRST_POLYGON, true);

        // let other_line_string = other_polygon.exterior().clone();
        // remove holes is the only help ???
        // let other_polygon = Polygon::new(other_line_string, vec![]);
        let self_polygon = self.to_geo_polygon(FIRST_POLYGON, false);
        //let self_line_string = self_polygon.exterior().clone();
        //println!("self: {:?}", self_polygon);
        //println!("other: {:?}", other_line_string);
        //let x = self_polygon.contains(&other_polygon);
        //let y = self_line_string.contains(&other_polygon);
        //let z = self_line_string.contains(&other_line_string);
        //println!("contains: {x} {y} {z}",);
        //x

        let (_out, _hol) = other_polygon.clone().into_inner();
        let self_exterior = self_polygon.exterior();
        let other_exterior = other_polygon.exterior();
        for (index, coord) in other_exterior.coords_iter().enumerate() {
            // MultiPoly with holes has more digits and so is not ON the line
            let on_line = self_exterior.contains(&coord);
            // for polygons, contains means NOT on the line
            let contains = self_exterior.contains(&coord);
            println!("{index} coord: {:?} {contains} {on_line}", coord);
            if !(on_line || contains) {
                return false;
            };
        }

        true
    }

    /**
     * Check if any point in a part is within this building's outline.
     * It only checknof points are inside, not if crossing events occur, or
     * if the part completly surrounds the building.
     * @param {BuildingPart} part - the part to be tested
     * @returns {bool} is it?
     */
    pub fn _other_is_inside(&self, other: &Footprint) -> bool {
        // println!("\ntttother:\n{:?}\n", other.polygons);
        // println!("\ntttself:\n{:?}\n", self.polygons);
        let other_outer = &other.polygons[FIRST_POLYGON][OUTER_POLYGON];
        //println!("tttttt: {:?}", &self.polygons[FIRST_POLYGON]);
        let self_outer = &self.polygons[FIRST_POLYGON][OUTER_POLYGON];

        for (_index, position) in other_outer.iter().enumerate() {
            //if index != 1 {
            //    continue; // ttt
            //}

            if !_ai_surrounds(
                // &self.polygons[FIRST_POLYGON][OUTER_POLYGON],
                &self_outer,
                position,
            ) {
                //println!("tt_3 index: {index} p: {:?}", position);
                return false;
            }
        }
        true
    }
}

/// Checks if a point is inside or on the border of a polygon.
/// `shape`: list of polygon vertices (must be a closed polygon, first and last point don't need to be the same)
/// `point`: the (x, y) point to check
pub fn _baker_surrounds(positions: &GroundPositions, point: &GroundPosition) -> bool {
    let mut count = 0;
    let n = positions.len();

    for (i, position) in positions.iter().enumerate() {
        //for i in 0..n {
        //let vec = positions[i];
        let next_pos = positions[(i + 1) % n];

        if position.east == point.east && position.north == point.north {
            return true; // Point is exactly on a vertex
        }

        if next_pos.east == position.east {
            // Vertical line
            if point.east == position.east {
                return true; // Point is on a vertical line
            }
            if position.east > point.east
                // West of vertical line and not abowe or below
                && (position.north > point.north || next_pos.north > point.north)
                && !(position.north > point.north && next_pos.north > point.north)
            {
                // went into or out of area
                count += 1;
            }
        } else if next_pos.north == position.north {
            // Horizontal line
            if position.north == point.north
                && (position.east > point.east || next_pos.east > point.east)
                && !(position.east > point.east && next_pos.east > point.east)
            {
                return true; // Point is on a horizontal edge
            }
        } else {
            // slopy line
            let slope = (next_pos.north - position.north) / (next_pos.east - position.east);
            let intercept = position.north - slope * position.east;
            let intersection = (point.north - intercept) / slope;

            println!(
                "slope: {slope} intercept: {intercept} intersection: {intersection} point.east: {:?}",
                point.east
            );
            println!(
                "max: {} {} min: {} {} next: {} this: {}",
                intersection < f32::max(next_pos.east, position.east),
                f32::max(next_pos.east, position.east),
                intersection > f32::min(next_pos.east, position.east),
                f32::min(next_pos.east, position.east),
                next_pos.east,
                position.east
            );

            // count how often the point is east of one of the lines
            if intersection > point.east
                && intersection < f32::max(next_pos.east, position.east)
                && intersection > f32::min(next_pos.east, position.east)
            {
                count += 1;
            } else if (intersection - point.east).abs() < f32::EPSILON {
                return true; // Point lies exactly on the edge
            }
        }
    }

    count % 2 == 1
}

// https://docs.rs/geo/latest/geo/
// https://github.com/georust/geo/blob/38afc3ed21f2c3e0abeb2658947bceab48b65102/geo/src/algorithm/contains/point.rs#L23

// Wikipedia: Diese Methode gibt true zurück, wenn der Punkt innerhalb des Polygons liegt, sonst false
fn _wiki_surrounds(polygon: &GroundPositions, point: &GroundPosition) -> bool {
    let mut is_inside: bool = false;

    for (i, _polygon) in polygon.iter().enumerate()
    // Diese for-Schleife durchläuft alle Ecken des Polygons
    //for (int i = 0; i < polygon.Length; i++)
    {
        let j = (i + 1) % polygon.len(); // Index der nächsten Ecke
        //println!("i/j: {i}/{j}");
        if polygon[i].north < point.north && polygon[j].north >= point.north
            || polygon[j].north < point.north && polygon[i].north >= point.north
        {
            println!("partly {i}");
            if (point.north - polygon[i].north) * (polygon[j].east - polygon[i].east)
                < (point.east - polygon[i].east) * (polygon[j].north - polygon[i].north)
            // Wenn der Strahl die Kante schneidet, Rückgabewert zwischen true und false wechseln
            {
                is_inside = !is_inside;
                println!("is_inside: {is_inside}");
            }
        }
    }
    println!("!!! is_inside: {is_inside}");
    is_inside
}

/// Checks if a point is inside a polygon using the ray casting algorithm.
/// ChatGPT generated code - does not work
///
/// # Arguments
/// * `point` - The point to test.
/// * `polygon` - A slice of points representing the polygon vertices, ordered clockwise or counterclockwise.
///
/// # Returns
/// * `true` if the point is inside the polygon, `false` otherwise.
fn _ai_surrounds(positions: &GroundPositions, point: &GroundPosition) -> bool {
    let mut inside = false;
    let n = positions.len();

    if n < 3 {
        return false; // Not a valid polygon
    }

    //let mut j = n - 1;
    for i in 0..n {
        let pi = positions[i];
        let pj = positions[(i + 1) % n]; // there was no modulo, only positions[j]

        let intersect = ((pi.north > point.north) != (pj.north > point.north))
            && (point.east
                < (pj.east - pi.east) * (point.north - pi.north)
                    / (pj.north - pi.north + 0.001) // f32::EPSILON)
                    + pi.east);

        if intersect {
            inside = !inside;
        }

        //j = i;
    }

    if !inside {
        println!("ai outside");
    }

    inside
}
