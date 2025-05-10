use triangulation::{Delaunay, Point};
//e triangulate::{self, formats, Polygon};
use crate::kernel_in::{BuildingPart, GroundPosition, RoofShape};
use crate::kernel_out::{GpuPosition, OsmMeshAttributes, RenderColor};

///////////////////////////////////////////////////////////////////////////////////////////////////
// OSM ////////////////////////////////////////////////////////////////////////////////////////////

// Constants / Parameters
static MULTI_MESH: bool = false;
static _GPU_POS_NULL: GpuPosition = [0.0, 0.0, 0.0];

pub fn scan_objects(building_parts: Vec<BuildingPart>) -> Vec<OsmMeshAttributes> {
    let mut osm_attributs = Vec::new();

    let mut osm_mesh = OsmMesh::new();
    for building_part in building_parts {
        osm_mesh.prepare_roof(&building_part);

        osm_mesh.push_building_part(&building_part);

        if MULTI_MESH {
            //println!("MULTI_MESH");
            osm_attributs.push(osm_mesh.attributes);
            osm_mesh = OsmMesh::new();
        }
    }

    if !MULTI_MESH {
        osm_attributs.push(osm_mesh.attributes);
    }

    osm_attributs
}

// Methode-Extenton of the "CLASS" OSM-Mesh, only needed internaly here ////////////////////////
struct OsmMesh {
    attributes: OsmMeshAttributes,
}

impl OsmMesh {
    pub fn new() -> Self {
        OsmMesh {
            attributes: OsmMeshAttributes::new(),
        }
    }

    pub fn push_building_part(&mut self, building_part: &BuildingPart) {
        let min_height = building_part.min_height;
        let wall_height = building_part.wall_height;
        let roof_height = building_part.roof_height;
        // println!("ttt m: {} w:{} r:{}", min_height, wall_height, roof_height);

        // https://docs.rs/geo/latest/geo/geometry/struct.LineString.html#impl-IsConvex-for-LineString%3CT%3E

        let color = building_part.color;
        let roof_color = building_part.roof_color;

        let mut roof_polygon: Vec<Point> = Vec::new();
        let mut roof_positions: Vec<GpuPosition> = Vec::new();

        //// Push all Walls ////
        let position = building_part.footprint.positions.last().unwrap();
        // todo: fn for next 3 lines
        let height = self.calc_roof_position_height(position, &building_part);
        let mut last_pos_down = position.to_gpu_position(min_height);
        let mut last_pos_up = position.to_gpu_position(height);

        // Whay rev() ?  Better chane push_square order ???
        for position in building_part.footprint.positions.iter() {
            let height = self.calc_roof_position_height(position, &building_part);
            let this_pos_down = position.to_gpu_position(min_height);
            let this_pos_up = position.to_gpu_position(height);

            // Walls
            self.push_square(
                last_pos_down,
                this_pos_down,
                last_pos_up,
                this_pos_up,
                color,
            );

            // Roof Points for triangulation and Onion, Positions for a Phyramide
            let roof_point = Point::new(position.east, position.north);
            roof_polygon.push(roof_point);
            roof_positions.push(this_pos_up);
            last_pos_down = this_pos_down;
            last_pos_up = this_pos_up;
        }

        match building_part.roof_shape {
            //
            RoofShape::Phyramidal => self.push_phyramid(
                building_part
                    .footprint
                    .center
                    .to_gpu_position(wall_height + roof_height),
                roof_positions,
                roof_color,
            ),

            RoofShape::Onion => self.push_onion(
                roof_polygon,
                building_part.footprint.center,
                wall_height,
                roof_height,
                roof_color,
            ),

            _ => self.push_flat(roof_positions, roof_polygon, roof_color),
        }
    }

    fn prepare_roof(&mut self, _building_part: &BuildingPart) {
        // println!("angle: {}", _building_part.roof_angle);
        // todo:
        // Add positions below roof first etc.
        // rotate a foodprint mirror
        // prepare height calculation
    }

    fn calc_skillion_position_height(
        &mut self,
        position: &GroundPosition,
        building_part: &BuildingPart,
    ) -> f32 {
        //println!("ph: position: {:?}", position);
        //        let roof_slope = circle_limit(building_part.roof_angle + f32::to_radians(90.));
        let east = position
            // why NOT - negativ??? (see other lines)
            .rotate_around_center(building_part.roof_angle, building_part.footprint.center)
            .east;
        let inclination = building_part.roof_height
            / (building_part.bounding_box_rotated.east - building_part.bounding_box_rotated.west); // Höhen/Tiefe der Nodes/Ecken berechenen

        // if (y >= -0.001) { // It's the roof, not the lower floor of the building(block)
        let height = building_part.wall_height + building_part.roof_height
        //  - f32::abs(east - building_part.footprint.center.east) * inclination;
            - f32::abs(east - building_part.bounding_box_rotated.west) * inclination;
        // !!: If the roof is "left" of the hightest side, it also must go down

        // ttt
        //println!(
        //    "ph: e {:?} i {:?} rh {:?} ew {:?} h {} ",
        //    east,
        //    inclination,
        //    building_part.roof_height,
        //    building_part.bounding_box_rotated.east - building_part.bounding_box_rotated.west,
        //    height,
        //);

        height
    }

    fn calc_roof_position_height(
        &mut self,
        position: &GroundPosition,
        building_part: &BuildingPart,
    ) -> f32 {
        match building_part.roof_shape {
            RoofShape::Skillion => self.calc_skillion_position_height(position, building_part),

            _ => building_part.wall_height,
        }
    }

    pub fn push_flat(
        &mut self,
        roof_positions: Vec<GpuPosition>,
        roof_polygon: Vec<Point>,
        color: RenderColor,
    ) {
        let roof_index_offset = self.attributes.vertices_positions.len();
        let triangulation = Delaunay::new(&roof_polygon).unwrap();
        let indices = triangulation.dcel.vertices;
        //println!("triangles: {:?}",&indices);
        // ? why .rev() ???
        for index in indices.iter().rev() {
            self.attributes
                .indices_to_vertices
                .push((roof_index_offset + index) as u32);
        }

        for position in roof_positions {
            self.attributes.vertices_positions.push(position);
            self.attributes.vertices_colors.push(color);
        }
    }

    // todo: phyramide, dome and onion the same except a different curves. Use same code,
    // todo: For all 3 shapetypes: less points: cornsers, much points: rounded
    pub fn push_phyramid(
        &mut self,
        pike: GpuPosition,
        roof_positions: Vec<GpuPosition>,
        color: RenderColor,
    ) {
        let roof_index_offset = self.attributes.vertices_positions.len();
        let pike_index_offset = roof_positions.len();
        for (index, position) in roof_positions.iter().enumerate() {
            self.attributes.vertices_positions.push(*position);
            self.attributes.vertices_colors.push(color);

            let index1 = index;
            let mut index2 = index + 1;
            if index2 >= roof_positions.len() {
                index2 = 0
            };
            self.push_indices([
                (roof_index_offset + index1),
                (roof_index_offset + index2),
                (roof_index_offset + pike_index_offset),
            ]);
        }
        self.attributes.vertices_positions.push(pike);
        self.attributes.vertices_colors.push(color);
        //println!("rio={} pio={} len={}",roof_index_offset, pike_index_offset,self.vertices_positions.len() );
    }

    pub fn push_onion(
        &mut self,
        roof_polygon: Vec<Point>,
        pike: GroundPosition,
        wall_height: f32,
        roof_height: f32,
        color: RenderColor,
    ) {
        let extrude_curve_scale = [
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

        let columns = roof_polygon.len();
        let one_column = columns * 2;

        // process all rings
        for scale in extrude_curve_scale {
            let scale_radius = scale[0] as f32;
            let scale_up = scale[1] as f32;
            //println!("scale {} {} {} {}",curve_up,curve_radius, one_column, roof_height);

            let edge_position = roof_polygon.last().unwrap();
            let gpu_x = (edge_position.x - pike.east) * scale_radius + pike.east;
            let gpu_z = (edge_position.y - pike.north) * scale_radius + pike.north; // * roof_rel
                                                                                    // todo 2*: use .to_GpuPosition
            let mut last_pos = [gpu_x, wall_height + roof_height * scale_up, -gpu_z];

            // process one ring
            for edge_position in roof_polygon.iter() {
                // push colors
                self.attributes.vertices_colors.push(color);
                self.attributes.vertices_colors.push(color);
                // push vertices
                let gpu_x = (edge_position.x - pike.east) * scale_radius + pike.east;
                let gpu_z = (edge_position.y - pike.north) * scale_radius + pike.north; // * roof_rel
                let this_pos = [gpu_x, wall_height + roof_height * scale_up, -gpu_z];

                // push indices
                let index = self.attributes.vertices_positions.len();
                self.attributes.vertices_positions.push(last_pos); // right vertice different than left to get corneres
                self.attributes.vertices_positions.push(this_pos); // left - up=down to not get corners
                last_pos = this_pos;
                //println!("pso x z {} {} {:?} {:?}",pos_x,pos_z,last_pos,this_pos);

                // not if it is the last point/ring of the curve
                if scale_radius > 0. {
                    // Push indices of two treeangles
                    self.push_indices([index, index + 1, index + one_column]);
                    self.push_indices([index + 1, index + one_column + 1, index + one_column]);
                }
            } // ring
        } // all rings
    } // OsmMesh

    pub fn push_square(
        &mut self,
        down_left: GpuPosition,
        down_right: GpuPosition,
        up_left: GpuPosition,
        up_right: GpuPosition,
        color: RenderColor,
    ) {
        // First index of the comming 4 positions
        let index = self.attributes.vertices_positions.len();

        // Push the for colors
        self.attributes.vertices_colors.push(color);
        self.attributes.vertices_colors.push(color);
        self.attributes.vertices_colors.push(color);
        self.attributes.vertices_colors.push(color);

        // Push the for positions
        self.attributes.vertices_positions.push(down_left); //  +0     2---3
        self.attributes.vertices_positions.push(down_right); // +1     |   |
        self.attributes.vertices_positions.push(up_left); //    +2     0---1
        self.attributes.vertices_positions.push(up_right); //   +3

        // Push first and second treeangle
        self.push_indices([index /*....*/, index + 1, index + 2]);
        self.push_indices([index /*.*/+ 1, index + 3, index + 2]);
    }

    pub fn push_indices(&mut self, indexi: [usize; 3]) {
        self.attributes.indices_to_vertices.push(indexi[0] as u32);
        self.attributes.indices_to_vertices.push(indexi[1] as u32);
        self.attributes.indices_to_vertices.push(indexi[2] as u32);
    }
}
