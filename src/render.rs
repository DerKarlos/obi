//e triangulate::{self, formats, Polygon};
use triangulation::{Delaunay, Point};

use crate::input_api::{BuildingOrPart, RenderColor, RoofShape};

// Bevy pbr color needs f32, The parse has no .to_f32_array???}
// https://docs.rs/csscolorparser/latest/csscolorparser/

///////////////////////////////////////////////////////////////////////////////////////////////////
// OSM ////////////////////////////////////////////////////////////////////////////////////////////

type Position = [f32; 3];
//type XzVec = [f32;2];

// Constants / Parameters
static MULTI_MESH: bool = false;
static GPU_POS_NULL: [f32; 3] = [0.0, 0.0, 0.0];

static DEFAULT_WALL_COLOR: [f32; 4] = [0.5, 0.5, 0.5, 1.0]; // "grey"
static DEFAULT_ROOF_COLOR: [f32; 4] = [1.0, 0.0, 0.0, 1.0]; // "red"

static DEFAULT_BUILDING_HEIGHT: f32 = 10.0;

pub fn scan_osm(buildings_or_parts: Vec<BuildingOrPart>) -> Vec<OsmMesh> {
    let mut osm_meshes = Vec::new();

    let mut osm_mesh = OsmMesh::new();

    for building_or_part in buildings_or_parts {
        let part_height = building_or_part.height.unwrap_or(DEFAULT_BUILDING_HEIGHT);
        let min_height = building_or_part.min_height.unwrap_or(0.0);
        let roof = building_or_part.roof.unwrap();
        let roof_height = roof.height.unwrap_or(0.0);
        // https://docs.rs/geo/latest/geo/geometry/struct.LineString.html#impl-IsConvex-for-LineString%3CT%3E

        let mut last_pos_down = GPU_POS_NULL;
        let mut last_pos_up = GPU_POS_NULL;
        let mut sum_east = 0.;
        let mut sum_north = 0.;
        let colour = building_or_part.color.unwrap_or(DEFAULT_WALL_COLOR);
        let roof_colour = roof.color.unwrap_or(DEFAULT_ROOF_COLOR);

        let mut roof_polygon: Vec<Point> = Vec::new();
        let mut roof_positions: Vec<[f32; 3]> = Vec::new();

        // The polygon node list is "closed": Last is connected to first
        for (index, position) in building_or_part.foodprint.iter().rev().enumerate() {
            let this_pos_down = [position.east, min_height, position.north];
            let this_pos_up = [position.east, part_height, position.north];
            let roof_point = Point::new(position.east, position.north);
            // skip first node = last
            if index > 0 {
                // Walls
                osm_mesh.push_square(
                    last_pos_down,
                    this_pos_down,
                    last_pos_up,
                    this_pos_up,
                    colour,
                );

                // Roof
                roof_polygon.push(roof_point);
                roof_positions.push(this_pos_up);
                sum_east += position.east;
                sum_north += position.north;
            }
            last_pos_down = this_pos_down;
            last_pos_up = this_pos_up;
        }
        // center of way
        const LAST_AS_IT_IS_EQUAL_TO_FIRST: usize = 1;
        let count = (building_or_part.foodprint.len() - LAST_AS_IT_IS_EQUAL_TO_FIRST) as f32;
        sum_east /= count;
        sum_north /= count;

        match roof.shape {
            //
            crate::input_api::RoofShape::Phyramidal => osm_mesh.push_pyramid(
                roof_positions,
                [sum_east, part_height + roof_height, sum_north],
                roof_colour,
            ),

            RoofShape::Onion => osm_mesh.push_onion(
                part_height,
                roof_height,
                roof_polygon,
                Point {
                    x: sum_east,
                    y: sum_north,
                },
                roof_colour,
            ),

            _ => osm_mesh.push_flat(roof_positions, roof_polygon, roof_colour),
        }

        if MULTI_MESH {
            //println!("MULTI_MESH");
            osm_meshes.push(osm_mesh);
            osm_mesh = OsmMesh::new();
        }
    }

    if !MULTI_MESH {
        osm_meshes.push(osm_mesh);
    }

    osm_meshes
}

// "CLASS" Custom Mesh /////////////////////////////////////////

pub struct OsmMesh {
    // todo: not pub but fn get
    pub colors: Vec<RenderColor>,         // format: Float32x4
    pub position_vertices: Vec<Position>, // 3 coordinates * x Positions. The corners are NOT reused to get hard Kanten
    pub indices: Vec<u32>,
}

impl OsmMesh {
    pub fn new() -> Self {
        Self {
            colors: vec![],
            position_vertices: vec![],
            indices: vec![],
        }
    }

    pub fn push_onion(
        &mut self,
        part_height: f32,
        roof_height: f32,
        roof_polygon: Vec<Point>,
        pike: Point,
        colour: [f32; 4],
    ) {
        let shape_curve = [
            // -x- |y|    The curve is about "taken" from F4map.com
            [1.00, 0.00],
            [1.12, 0.09],
            [1.27, 0.15],
            [1.36, 0.27],
            [1.28, 0.42],
            [1.10, 0.51],
            [0.95, 0.53],
            [0.62, 0.58],
            [0.49, 0.61],
            [0.21, 0.69],
            [0.10, 0.79],
            [0.00, 1.00],
        ];

        let columns = roof_polygon.len() as i32;
        let to_next_column = columns * 2;

        for point in shape_curve {
            // process all rings

            let curve_radius = point[0] as f32;
            let curve_up = point[1] as f32;
            //println!("scale {} {} {} {}",curve_up,curve_radius, to_next_column, roof_height);

            let column_point = roof_polygon.last().unwrap();
            let pos_x = (column_point.x - pike.x) * curve_radius + pike.x;
            let pos_z = (column_point.y - pike.y) * curve_radius + pike.y; // * roof_rel
            let mut last_pos = [pos_x, part_height + roof_height * curve_up, pos_z];

            for column_point in roof_polygon.iter() {
                // process one ring

                // push colors
                self.colors.push(colour);
                self.colors.push(colour);

                // push vertices
                let pos_x = (column_point.x - pike.x) * curve_radius + pike.x;
                let pos_z = (column_point.y - pike.y) * curve_radius + pike.y;
                let this_pos = [pos_x, part_height + roof_height * curve_up, pos_z];
                //println!("pso x z {} {} {:?} {:?}",pos_x,pos_z,last_pos,this_pos);
                let index = self.position_vertices.len() as i32;
                self.position_vertices.push(last_pos); // right vertice different than left to get corneres
                self.position_vertices.push(this_pos); // left - up=down to not get corners
                last_pos = this_pos;

                if curve_radius > 0. {
                    // not if it is the last point/ring of the curve
                    // Push indices, first treeangle
                    self.indices.push(index as u32); // 0     2
                    self.indices.push((index + 1) as u32); // 1     | \
                    self.indices.push((index + to_next_column) as u32); // 2     0---1
                                                                        // Secound treeangle
                    self.indices.push((index + 1) as u32); // 0     2---1
                    self.indices.push((index + to_next_column + 1) as u32); // 1       \ |
                    self.indices.push((index + to_next_column) as u32); // 2         0
                                                                        //println!("index {} {}",index,index+to_next_column);
                }
            } // ring
        } // all rings
    } // OsmMesh

    pub fn push_flat(
        &mut self,
        positions: Vec<Position>,
        roof_polygon: Vec<Point>,
        colour: [f32; 4],
    ) {
        let triangulation = Delaunay::new(&roof_polygon).unwrap();
        //println!("triangles: {:?}",&triangulation.dcel.vertices);
        let indices = triangulation.dcel.vertices;

        let roof_index_offset = self.position_vertices.len();
        for position in positions {
            self.position_vertices.push(position);
            self.colors.push(colour);
        }
        for index in indices {
            self.indices.push((roof_index_offset + index) as u32);
        }
    }

    pub fn push_pyramid(&mut self, positions: Vec<Position>, pike: Position, colour: [f32; 4]) {
        let roof_index_offset = self.position_vertices.len();
        let pike_index_offset = positions.len();
        for (index, position) in positions.iter().enumerate() {
            self.position_vertices.push(*position);
            self.colors.push(colour);

            let index1 = index;
            let mut index2 = index + 1;
            if index2 >= positions.len() {
                index2 = 0
            }
            self.indices.push((roof_index_offset + index1) as u32);
            self.indices.push((roof_index_offset + index2) as u32);
            self.indices
                .push((roof_index_offset + pike_index_offset) as u32);
        }
        self.position_vertices.push(pike);
        self.colors.push(colour);
        //println!("ttt rio={} pio={} len={}",roof_index_offset, pike_index_offset,self.position_vertices.len() );
    }

    pub fn push_square(
        &mut self,
        down_left: Position,
        down_right: Position,
        up_left: Position,
        up_right: Position,
        colour: [f32; 4],
    ) {
        const O: usize = 0; // To make the columns nice,  + 0 gets a clippy warning
                            // First index of the comming 4 positions
        let index = self.position_vertices.len();
        // Push the for colors
        self.colors.push(colour);
        self.colors.push(colour);
        self.colors.push(colour);
        self.colors.push(colour);
        // Push the for positions
        self.position_vertices.push(down_left); //  +0     2---3
        self.position_vertices.push(down_right); // +1     |   |
        self.position_vertices.push(up_left); //    +2     0---1
        self.position_vertices.push(up_right); //   +3
                                               // First treeangle
        self.indices.push((index + O) as u32); //     2
        self.indices.push((index + 1) as u32); //     | \
        self.indices.push((index + 2) as u32); //     0---1
                                               // Secound treeangle
        self.indices.push((index + 1) as u32); //     2---3
        self.indices.push((index + 3) as u32); //       \ |
        self.indices.push((index + 2) as u32); //         1
    }
}
