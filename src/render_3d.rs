use crate::kernel_in::{BuildingPart, GroundPosition, RoofShape};
use crate::kernel_out::{GpuPosition, OsmMeshAttributes, RenderColor};
use crate::shape::Shape;
use std::ops::Sub;

///////////////////////////////////////////////////////////////////////////////////////////////////
// OSM ////////////////////////////////////////////////////////////////////////////////////////////

// Constants / Parameters
static MULTI_MESH: bool = false;
static _GPU_POSITION_NULL: GpuPosition = [0.0, 0.0, 0.0];

// Local methodes of GroundPosition, only to be used in the renderer!
impl GroundPosition {
    fn to_gpu_position(&self, height: f32) -> GpuPosition {
        // Minus north because +north is -z in the GPU space.
        [self.east, height, -self.north]
    }
}

impl Shape {
    fn get_gpu_positions(&self, height: f32) -> Vec<GpuPosition> {
        let mut roof_gpu_positions: Vec<GpuPosition> = Vec::new();
        for position in &self.positions {
            let this_gpu_position_up = position.to_gpu_position(height);
            roof_gpu_positions.push(this_gpu_position_up);
        }
        roof_gpu_positions
    }
}

pub fn scan_objects(building_parts: Vec<BuildingPart>) -> Vec<OsmMeshAttributes> {
    let mut osm_attributs = Vec::new();

    let mut osm_mesh = OsmMesh::new();
    for mut building_part in building_parts {
        osm_mesh.prepare_roof(&building_part);

        osm_mesh.push_building_part(&mut building_part);

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
#[derive(Clone, Debug)]
struct OsmMesh {
    attributes: OsmMeshAttributes,
}

impl OsmMesh {
    fn new() -> Self {
        OsmMesh {
            attributes: OsmMeshAttributes::new(),
        }
    }

    fn push_building_part(&mut self, building_part: &mut BuildingPart) {
        let min_height = building_part.min_height;
        let wall_height = building_part.wall_height;
        let roof_height = building_part.roof_height;
        // println!("ttt m: {} w:{} r:{}", min_height, wall_height, roof_height);

        // https://docs.rs/geo/latest/geo/geometry/struct.LineString.html#impl-IsConvex-for-LineString%3CT%3E

        let color = building_part.color;
        let roof_color = building_part.roof_color;

        match building_part.roof_shape {
            //
            RoofShape::Skillion => {
                self.push_skillion(&building_part, roof_color);
            }

            RoofShape::Gabled => {
                self.push_gabled(building_part, roof_color);
            }

            RoofShape::Phyramidal => self.push_phyramid(
                &building_part.footprint,
                wall_height,
                roof_height,
                roof_color,
            ),

            // todo: cupola
            //
            RoofShape::Onion => self.push_onion(
                &building_part.footprint,
                wall_height,
                roof_height,
                roof_color,
            ),

            _ => self.push_flat(&building_part.footprint, wall_height, roof_color),
        }

        self.push_walls(building_part, min_height, color);
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
            .sub(building_part.footprint.center)
            .rotate(-building_part.roof_angle) // skillion
            .east;
        println!("roof_angle: {}", building_part.roof_angle);
        let inclination = building_part.roof_height
            / (building_part.bounding_box_rotated.east - building_part.bounding_box_rotated.west); // Höhen/Tiefe der Nodes/Ecken berechenen

        let height = building_part.wall_height + building_part.roof_height
            - f32::abs(east - building_part.bounding_box_rotated.west) * inclination;

        height
    }

    fn calc_gabled_position_height(
        &mut self,
        position: &GroundPosition,
        building_part: &BuildingPart,
    ) -> f32 {
        let east = position
            .sub(building_part.footprint.center)
            .rotate(-building_part.roof_angle) // Rotate against the actual angle to got 0 degrees
            .east
            + building_part.footprint.shift;

        let width =
            building_part.bounding_box_rotated.east - building_part.bounding_box_rotated.west;
        let inclination = building_part.roof_height * 2. / width;

        let height =
            building_part.wall_height + building_part.roof_height - f32::abs(east) * inclination;

        let rh = building_part.roof_height;
        println!(
            "tttt East: {east} width: {width} roof_height: {rh} width: {width} Inc: {inclination} height: {height}"
        );

        height
    }

    fn calc_roof_position_height(
        &mut self,
        position: &GroundPosition,
        building_part: &BuildingPart,
    ) -> f32 {
        match building_part.roof_shape {
            RoofShape::Skillion => self.calc_skillion_position_height(position, building_part),
            RoofShape::Gabled => self.calc_gabled_position_height(position, building_part),

            _ => building_part.wall_height,
        }
    }

    fn push_flat(&mut self, footprint: &Shape, height: f32, color: RenderColor) {
        let mut roof_gpu_positions = footprint.get_gpu_positions(height);
        let roof_index_offset = self.attributes.vertices_positions.len();
        let indices = footprint.get_triangulate_indices();
        // println!("triangles: {:?}", &indices);

        // ? why .rev() ???
        for index in indices.iter().rev() {
            self.attributes
                .indices_to_vertices
                .push((roof_index_offset + index) as u32);
        }

        for _p in &roof_gpu_positions {
            self.attributes.vertices_colors.push(color);
        }

        self.attributes
            .vertices_positions
            .append(&mut roof_gpu_positions);
    }

    fn push_skillion(&mut self, building_part: &BuildingPart, color: RenderColor) {
        let footprint = &building_part.footprint;
        let mut roof_gpu_positions: Vec<GpuPosition> = Vec::new();
        for position in footprint.positions.iter() {
            let height = self.calc_roof_position_height(position, &building_part);
            roof_gpu_positions.push(position.to_gpu_position(height))
        }

        // let mut roof_gpu_positions = footprint.get_gpu_positions(height);
        let roof_index_offset = self.attributes.vertices_positions.len();
        let indices = footprint.get_triangulate_indices();
        // println!("triangles: {:?}", &indices);

        // ? why .rev() ?  see negativ???
        for index in indices.iter().rev() {
            self.attributes
                .indices_to_vertices
                .push((roof_index_offset + index) as u32);
        }

        for _p in &roof_gpu_positions {
            self.attributes.vertices_colors.push(color);
        }

        self.attributes
            .vertices_positions
            .append(&mut roof_gpu_positions);
    }

    fn push_gabled(&mut self, building_part: &mut BuildingPart, color: RenderColor) {
        let (face1, face2) = building_part
            .footprint
            .split_at_x_zero(building_part.roof_angle);

        self.push_roof_face(face1, color, building_part);
        self.push_roof_face(face2, color, building_part);
    }

    fn push_roof_face(
        &mut self,
        side: Vec<GroundPosition>,
        color: RenderColor,
        building_part: &BuildingPart,
    ) {
        let mut footprint = Shape::new(); // &building_part.footprint;
        footprint.positions = side;

        let roof_index_offset = self.attributes.vertices_positions.len();
        let indices = footprint.get_triangulate_indices();
        println!("offset: {roof_index_offset}  indices: {:?}", &indices);

        // ? why .rev() ?  see negativ???
        for index in indices.iter().rev() {
            self.attributes
                .indices_to_vertices
                .push((roof_index_offset + index) as u32);
        }

        for position in footprint.positions.iter() {
            let height = self.calc_roof_position_height(position, &building_part);
            self.attributes
                .vertices_positions
                .push(position.to_gpu_position(height))
        }

        for _position in &footprint.positions {
            self.attributes.vertices_colors.push(color);
        }

        //println!("attr: {:?}", &self.attributes);
        // attr: OsmMeshAttributes {
        //   indices_to_vertices: [1, 2, 3, 3, 0, 1],
        //   vertices_colors: [[0.54509807, 0.0, 0.0, 1.0], [0.54509807, 0.0, 0.0, 1.0], [0.54509807, 0.0, 0.0, 1.0], [0.54509807, 0.0, 0.0, 1.0]],
        //   vertices_positions: [] }
    }

    // todo: phyramide, dome and onion the same except a different curves. Use same code,
    // todo: For all 3 shapetypes: less points: cornsers, much points: rounded
    fn push_phyramid(
        &mut self,
        footprint: &Shape,
        wall_height: f32,
        roof_height: f32,
        color: RenderColor,
    ) {
        let pike = footprint.center.to_gpu_position(wall_height + roof_height);

        let roof_gpu_positions = footprint.get_gpu_positions(wall_height);
        let roof_index_offset = self.attributes.vertices_positions.len();
        let pike_index_offset = roof_gpu_positions.len();
        for (index, position) in roof_gpu_positions.iter().enumerate() {
            self.attributes.vertices_positions.push(*position);
            self.attributes.vertices_colors.push(color);

            let index1 = index;
            let mut index2 = index + 1;
            if index2 >= roof_gpu_positions.len() {
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

    fn push_onion(
        &mut self,
        footprint: &Shape,
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

        let columns = footprint.positions.len();
        let one_column = columns * 2;

        // process all rings
        for scale in extrude_curve_scale {
            let scale_radius = scale[0] as f32;
            let scale_up = scale[1] as f32;
            //println!("scale {} {} {} {}",curve_up,curve_radius, one_column, roof_height);

            let edge_position = footprint.positions.last().unwrap();
            let pike = footprint.center;
            let gpu_x = (edge_position.east - pike.east) * scale_radius + pike.east;
            let gpu_z = (edge_position.north - pike.north) * scale_radius + pike.north; // * roof_rel
                                                                                        // todo 2*: use .to_GpuPosition
            let mut last_pos = [gpu_x, wall_height + roof_height * scale_up, -gpu_z];

            // process one ring
            for edge_position in footprint.positions.iter() {
                // push colors
                self.attributes.vertices_colors.push(color);
                self.attributes.vertices_colors.push(color);
                // push vertices
                let gpu_x = (edge_position.east - pike.east) * scale_radius + pike.east;
                let gpu_z = (edge_position.north - pike.north) * scale_radius + pike.north; // * roof_rel
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
    }

    fn push_walls(
        &mut self,
        building_part: &mut BuildingPart,
        min_height: f32,
        color: RenderColor,
    ) {
        let position = building_part.footprint.positions.last().unwrap();
        // todo: fn for next 3 lines
        let height = self.calc_roof_position_height(position, &building_part);
        let mut last_gpu_position_down = position.to_gpu_position(min_height);
        let mut last_gpu_position_up = position.to_gpu_position(height);

        for position in building_part.footprint.positions.iter() {
            let height = self.calc_roof_position_height(position, &building_part);
            let this_gpu_position_down = position.to_gpu_position(min_height);
            let this_gpu_position_up = position.to_gpu_position(height);

            // Walls
            self.push_square(
                last_gpu_position_down,
                this_gpu_position_down,
                last_gpu_position_up,
                this_gpu_position_up,
                color,
            );

            // Roof Points for triangulation and Onion, Positions for a Phyramide
            last_gpu_position_down = this_gpu_position_down;
            last_gpu_position_up = this_gpu_position_up;
        }
    }

    //// basic pushes: ////

    fn push_square(
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

    fn push_indices(&mut self, indexi: [usize; 3]) {
        self.attributes.indices_to_vertices.push(indexi[0] as u32);
        self.attributes.indices_to_vertices.push(indexi[1] as u32);
        self.attributes.indices_to_vertices.push(indexi[2] as u32);
    }
}
