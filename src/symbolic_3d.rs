use crate::footprint::Footprint;
use crate::kernel_in::{
    BuildingOrPart, BuildingsAndParts, GroundPosition, GroundPositions, RoofShape,
};
use crate::kernel_out::{OsmMeshAttributes, RenderColor, RenderPosition};
use std::cmp::min;
use std::ops::Sub;

///////////////////////////////////////////////////////////////////////////////////////////////////
// OSM ////////////////////////////////////////////////////////////////////////////////////////////

// Constants / Parameters
static MULTI_MESH: bool = false;
static _GPU_POSITION_NULL: RenderPosition = [0.0, 0.0, 0.0];
static O: usize = 0; // Just to silent lint, make some lines equal and to show, the Offset may also be 0

// Local methodes of GroundPosition, only to be used in the renderer!
impl GroundPosition {
    pub fn to_gpu_position(self, height: f32) -> RenderPosition {
        // Minus north because +north is -z in the GPU space.
        [self.east, height, -self.north]
    }
}

impl Footprint {
    fn get_gpu_positions(&self, polygon_index: usize, height: f32) -> Vec<RenderPosition> {
        let mut roof_gpu_positions: Vec<RenderPosition> = Vec::new();
        for position in &self.polygons[polygon_index][0] {
            let this_gpu_position_up = position.to_gpu_position(height);
            roof_gpu_positions.push(this_gpu_position_up);
        }

        if self.polygons[polygon_index].len() > 1 {
            for hole_index in 1..self.polygons[polygon_index].len() {
                let hole: &GroundPositions = &self.first_polygon_u()[hole_index];
                for position in hole {
                    let this_gpu_position_up = position.to_gpu_position(height);
                    roof_gpu_positions.push(this_gpu_position_up);
                }
            }
        }

        roof_gpu_positions
    }
}

pub fn scan_elements_from_layer_to_mesh(
    buildings_and_parts: BuildingsAndParts,
) -> Vec<OsmMeshAttributes> {
    let mut osm_attributs = Vec::new();

    let mut osm_mesh = OsmMesh::new();
    for mut building_or_part in buildings_and_parts {
        osm_mesh.prepare_roof(&building_or_part);

        osm_mesh.push_building_or_part(&mut building_or_part);

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

    fn push_building_or_part(&mut self, building_or_part: &mut BuildingOrPart) {
        let min_height = building_or_part.min_height;
        let wall_height = building_or_part.wall_height;
        let roof_height = building_or_part.roof_height;
        // println!("- m: {} w:{} r:{}", min_height, wall_height, roof_height);

        // https://docs.rs/geo/latest/geo/geometry/struct.LineString.html#impl-IsConvex-for-LineString%3CT%3E

        let color = building_or_part.building_color;
        let roof_color = building_or_part.roof_color;

        if building_or_part.footprint.polygons[0][0].is_empty() {
            //println!("footprint.positions.is_empty: {}", building_or_part.id);
            return; // after parts subtractions, nothing is left
        }

        match building_or_part.roof_shape {
            //
            RoofShape::Skillion => {
                self.push_skillion(building_or_part, roof_color);
            }

            RoofShape::Gabled => {
                self.push_gabled(building_or_part, roof_color);
            }

            RoofShape::Phyramidal => self.push_phyramid(
                &building_or_part.footprint,
                wall_height,
                roof_height,
                roof_color,
            ),

            RoofShape::Dome => self.push_dome(
                &building_or_part.footprint,
                wall_height,
                roof_height,
                roof_color,
            ),

            RoofShape::Onion => self.push_onion(
                &building_or_part.footprint,
                wall_height,
                roof_height,
                roof_color,
            ),

            _ => self.push_flat(&mut building_or_part.footprint, wall_height, roof_color),
        }

        self.push_walls(building_or_part, min_height, color);
    }

    fn prepare_roof(&mut self, _: &BuildingOrPart) {
        // println!("angle: {}", _building_part.roof_angle);
        // todo:
        // Add positions below roof first etc.
        // rotate a foodprint mirror
        // prepare height calculation
    }

    fn calc_skillion_position_height(
        &mut self,
        position: &GroundPosition,
        building_or_part: &BuildingOrPart,
    ) -> f32 {
        //println!("ph: position: {:?}", position);
        //        let roof_slope = circle_limit(building_or_part.roof_angle + f32::to_radians(90.));
        let east = position
            .sub(building_or_part.footprint.center)
            .rotate(-building_or_part.roof_angle) // skillion
            .north;
        // println!("roof_angle: {}", building_or_part.roof_angle.to_degrees());
        let inclination = building_or_part.roof_height
            / (building_or_part.bounding_box_rotated.north
                - building_or_part.bounding_box_rotated.south); // Höhen/Tiefe der Nodes/Ecken berechenen

        building_or_part.wall_height + building_or_part.roof_height
            - f32::abs(east - building_or_part.bounding_box_rotated.south) * inclination
    }

    fn calc_gabled_position_height(
        &mut self,
        position: &GroundPosition,
        building_or_part: &BuildingOrPart,
    ) -> f32 {
        let east = position
            .sub(building_or_part.footprint.center)
            .rotate(-building_or_part.roof_angle) // Rotate against the actual angle to got 0 degrees
            .north
            + building_or_part.footprint.shift;

        let width = building_or_part.bounding_box_rotated.north
            - building_or_part.bounding_box_rotated.south;
        let inclination = building_or_part.roof_height * 2. / width;

        building_or_part.wall_height + building_or_part.roof_height - f32::abs(east) * inclination
    }

    fn _calc_skillion_position_height(
        &mut self,
        position: &GroundPosition,
        building_or_part: &BuildingOrPart,
    ) -> f32 {
        //println!("ph: position: {:?}", position);
        //        let roof_slope = circle_limit(building_or_part.roof_angle + f32::to_radians(90.));
        let east = position
            .sub(building_or_part.footprint.center)
            .rotate(-building_or_part.roof_angle) // skillion
            .east;
        // println!("roof_angle: {}", building_or_part.roof_angle.to_degrees());
        let inclination = building_or_part.roof_height
            / (building_or_part.bounding_box_rotated.east
                - building_or_part.bounding_box_rotated.west); // Höhen/Tiefe der Nodes/Ecken berechenen

        building_or_part.wall_height + building_or_part.roof_height
            - f32::abs(east - building_or_part.bounding_box_rotated.west) * inclination
    }

    fn _calc_gabled_position_height(
        &mut self,
        position: &GroundPosition,
        building_or_part: &BuildingOrPart,
    ) -> f32 {
        let east = position
            .sub(building_or_part.footprint.center)
            .rotate(-building_or_part.roof_angle) // Rotate against the actual angle to got 0 degrees
            .east
            + building_or_part.footprint.shift;

        let width =
            building_or_part.bounding_box_rotated.east - building_or_part.bounding_box_rotated.west;
        let inclination = building_or_part.roof_height * 2. / width;

        // let height =
        building_or_part.wall_height + building_or_part.roof_height - f32::abs(east) * inclination

        //3 let rh = building_or_part.roof_height;
        //3 println!(
        //3     "gabled - East: {east} width: {width} roof_height: {rh} width: {width} Inc: {inclination} height: {height}"
        //3 );

        //height
    }

    fn calc_roof_position_height(
        &mut self,
        position: &GroundPosition,
        building_or_part: &BuildingOrPart,
    ) -> f32 {
        match building_or_part.roof_shape {
            RoofShape::Skillion => self.calc_skillion_position_height(position, building_or_part),
            RoofShape::Gabled => self.calc_gabled_position_height(position, building_or_part),

            _ => building_or_part.wall_height,
        }
    }

    fn push_flat(&mut self, footprint: &mut Footprint, height: f32, color: RenderColor) {
        for polygon_index in 0..footprint.polygons.len() {
            let mut roof_gpu_positions = footprint.get_gpu_positions(polygon_index, height);
            let roof_index_offset = self.attributes.vertices_positions.len();
            let indices = footprint.get_triangulate_indices(polygon_index);
            // println!("triangles: {:?}", &indices);
            if indices.is_empty() {
                footprint.polygons[polygon_index] = Vec::new();
                continue;
            }

            // ? why .rev()?  see negativ ???
            for index in indices.iter().rev() {
                self.attributes
                    .indices_to_vertices
                    .push((roof_index_offset + index) as u32);
            }

            for _ in &roof_gpu_positions {
                self.attributes.vertices_colors.push(color);
            }

            self.attributes
                .vertices_positions
                .append(&mut roof_gpu_positions);
        }
    }

    fn push_skillion(&mut self, building_or_part: &mut BuildingOrPart, color: RenderColor) {
        for polygon_index in 0..building_or_part.footprint.polygons.len() {
            let footprint = &building_or_part.footprint;
            let mut roof_gpu_positions: Vec<RenderPosition> = Vec::new();
            for position in footprint.polygons[polygon_index][0].iter() {
                let height = self.calc_roof_position_height(position, building_or_part);
                roof_gpu_positions.push(position.to_gpu_position(height))
            }

            let roof_index_offset = self.attributes.vertices_positions.len();
            let indices = footprint.get_triangulate_indices(polygon_index);
            // println!("triangles: {:?}", &indices);
            if indices.is_empty() {
                building_or_part.footprint.polygons[polygon_index] = Vec::new();
                continue;
            }

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
    }

    fn push_gabled(&mut self, building_or_part: &mut BuildingOrPart, color: RenderColor) {
        let (face1, face2) = building_or_part
            .footprint
            .split_at_y_zero(building_or_part.roof_angle);

        self.push_roof_shape(face1, color, building_or_part);
        self.push_roof_shape(face2, color, building_or_part);
    }

    fn push_roof_shape(
        &mut self,
        side: GroundPositions,
        color: RenderColor,
        building_or_part: &BuildingOrPart,
    ) {
        for _polygon_index in 0..building_or_part.footprint.polygons.len() {
            // todo: gabled for cutted buildings
        }

        let mut footprint = Footprint::new(4712); // &building_or_part.footprint;
        footprint.polygons[0][0] = side;

        let roof_index_offset = self.attributes.vertices_positions.len();
        let indices = footprint.get_triangulate_indices(0);
        //println!("offset: {roof_index_offset}  indices: {:?}", &indices);
        if indices.is_empty() {
            return;
        }

        // ? why .rev() ?  see negativ ???
        for index in indices.iter().rev() {
            self.attributes
                .indices_to_vertices
                .push((roof_index_offset + index) as u32);
        }

        for position in footprint.polygons[0][0].iter() {
            let height = self.calc_roof_position_height(position, building_or_part);
            self.attributes
                .vertices_positions
                .push(position.to_gpu_position(height))
        }

        for _position in &footprint.polygons[0][0] {
            self.attributes.vertices_colors.push(color);
        }
    }

    fn push_phyramid(
        &mut self,
        footprint: &Footprint,
        wall_height: f32,
        roof_height: f32,
        color: RenderColor,
    ) {
        let ring_edges: Vec<ExtrudeRing> = vec![
            //todo: er!(1., 0.),
            ExtrudeRing {
                radius: 1.,
                height: 0.,
            },
            ExtrudeRing {
                radius: 0.,
                height: 1.,
            },
        ];
        let silhouette = Silhouette { ring_edges };
        self.push_extrude(footprint, silhouette, wall_height, roof_height, color);
    }

    fn push_dome(
        &mut self,
        footprint: &Footprint,
        wall_height: f32,
        roof_height: f32,
        color: RenderColor,
    ) {
        let mut ring_edges: Vec<ExtrudeRing> = Vec::new();
        const STEPS: usize = 10;
        for step in 0..STEPS {
            let angle = f32::to_radians((step * STEPS) as f32);
            ring_edges.push(ExtrudeRing {
                radius: angle.cos(),
                height: angle.sin(),
            });
            // println!("{step} a: {angle} {} {}", angle.cos(), angle.sin());
        }
        let silhouette = Silhouette { ring_edges };
        // println!("w: {wall_height} r: {roof_height}");
        self.push_extrude(footprint, silhouette, wall_height, roof_height, color);
    }

    fn calc_extrude_position(
        &mut self,
        ring: &ExtrudeRing,
        edge: &GroundPosition,
        wall_height: f32,
        roof_height: f32,
        pike: GroundPosition,
    ) -> RenderPosition {
        let gpu_x = (edge.east - pike.east) * ring.radius + pike.east;
        let gpu_z = (edge.north - pike.north) * ring.radius + pike.north;
        let gpu_y = wall_height + roof_height * ring.height;
        [gpu_x, gpu_y, -gpu_z] // Why -z?  Should be in an extra fn!
    }

    fn push_extrude(
        &mut self,
        footprint: &Footprint,
        silhouette: Silhouette,
        wall_height: f32,
        roof_height: f32,
        color: RenderColor,
    ) {
        let soft_edges = footprint.polygons[0][0].len() > 8;
        let mut gpu_positions: Vec<Vec<RenderPosition>> = Vec::new();
        for (ring_index, ring) in silhouette.ring_edges.iter().enumerate() {
            gpu_positions.push(Vec::new());
            for edge in footprint.polygons[0][0].iter() {
                gpu_positions[ring_index].push(self.calc_extrude_position(
                    ring,
                    edge,
                    wall_height,
                    roof_height,
                    footprint.center,
                ));
            }
        }

        const ONE_LESS_RING_FACES_BUT_RING_EDGES: usize = 1;
        let edges = footprint.polygons[0][0].len();
        let rings = silhouette.ring_edges.len() - ONE_LESS_RING_FACES_BUT_RING_EDGES;

        let start_index = self.attributes.vertices_positions.len();
        let pike_index = self.attributes.vertices_positions.len() + rings * edges;

        for ring_index in 0..rings {
            for edge_index in 0..edges {
                //println!("r: {ring_index} e: {edge_index}");

                if soft_edges {
                    self.push_soft_edges(
                        &gpu_positions,
                        ring_index,
                        edge_index,
                        edges,
                        start_index,
                        pike_index,
                        color,
                    );
                } else {
                    self.push_hard_edges(&gpu_positions, ring_index, edge_index, edges, color);
                }
            }
        }

        // push pike
        if soft_edges {
            //println!("gpu_positions: {:?}", gpu_positions);
            //println!("attributes: {:?}", self.attributes);
            let pike = gpu_positions[rings][0];
            self.attributes.vertices_positions.push(pike);
            self.attributes.vertices_colors.push(color);
        }
        // println!("self.attributes: {:?}", self.attributes);
    }

    fn push_soft_edges(
        &mut self,
        gpu_positions: &[Vec<RenderPosition>], // &Vec<Vec<RenderPosition>>,
        ring_index: usize,
        edge_index: usize,
        ec: usize, // edge count per ring
        start_index: usize,
        pike_index: usize,
        color: RenderColor,
    ) {
        let down_left = gpu_positions[ring_index][edge_index];
        self.attributes.vertices_positions.push(down_left);
        self.attributes.vertices_colors.push(color);

        // Calculate indexi of the square
        let index00 = (edge_index + O) % ec + (ring_index + O) * ec;
        let index10 = (edge_index + 1) % ec + (ring_index + O) * ec;
        let index01 = (edge_index + O) % ec + (ring_index + 1) * ec;
        let index11 = (edge_index + 1) % ec + (ring_index + 1) * ec;

        //println!(
        //    "10: {index10} {edge_index} {ec} {ring_index} {}",
        //    (edge_index + 1) % ec,
        //);
        // Push indices of two treeangles
        self.push_3_indices([
            min(start_index + index00, pike_index),
            min(start_index + index10, pike_index),
            min(start_index + index01, pike_index),
        ]);
        self.push_3_indices([
            min(start_index + index10, pike_index),
            min(start_index + index11, pike_index),
            min(start_index + index01, pike_index),
        ]);
    }

    fn push_hard_edges(
        &mut self,
        gpu_positions: &[Vec<RenderPosition>], //&Vec<Vec<RenderPosition>>,
        ring_index: usize,
        edge_index: usize,
        edges_count: usize,
        color: RenderColor,
    ) {
        let right = (edge_index + 1) % edges_count;
        let down_left = gpu_positions[ring_index][edge_index];
        let down_right = gpu_positions[ring_index][right];
        let up_left = gpu_positions[ring_index + 1][edge_index];
        let up_right = gpu_positions[ring_index + 1][right];

        self.push_square(down_left, down_right, up_left, up_right, color);
    }

    fn push_onion(
        &mut self,
        footprint: &Footprint,
        wall_height: f32,
        roof_height: f32,
        color: RenderColor,
    ) {
        let ring_edges: Vec<ExtrudeRing> = vec![
            ExtrudeRing {
                radius: 1.00,
                height: 0.00,
            },
            ExtrudeRing {
                radius: 1.12,
                height: 0.09,
            },
            ExtrudeRing {
                radius: 1.27,
                height: 0.15,
            },
            ExtrudeRing {
                radius: 1.36,
                height: 0.27,
            },
            ExtrudeRing {
                radius: 1.28,
                height: 0.42,
            },
            ExtrudeRing {
                radius: 1.10,
                height: 0.51,
            },
            ExtrudeRing {
                radius: 0.95,
                height: 0.53,
            },
            ExtrudeRing {
                radius: 0.62,
                height: 0.58,
            },
            ExtrudeRing {
                radius: 0.49,
                height: 0.61,
            },
            ExtrudeRing {
                radius: 0.21,
                height: 0.69,
            },
            ExtrudeRing {
                radius: 0.10,
                height: 0.79,
            },
            ExtrudeRing {
                radius: 0.00,
                height: 1.00,
            },
        ];

        let silhouette = Silhouette { ring_edges };
        self.push_extrude(footprint, silhouette, wall_height, roof_height, color);
    }

    fn push_walls(
        &mut self,
        building_or_part: &mut BuildingOrPart,
        min_height: f32,
        color: RenderColor,
    ) {
        let footprint = &building_or_part.footprint.clone();
        for polygon in &footprint.polygons {
            if polygon.is_empty() {
                continue;
            }
            let outer = &polygon[0].clone();
            self.push_wall_shape(
                building_or_part,
                outer,
                building_or_part.footprint.is_circular,
                min_height,
                color,
            );

            for hole_index in 1..polygon.len() {
                let hole: &GroundPositions = &polygon[hole_index];
                self.push_wall_shape(
                    building_or_part,
                    hole,
                    footprint.is_circular,
                    min_height,
                    color,
                );
            }
        }

        /******** /
        let outer = &footprint.polygons[0][0].clone();
        self.push_wall_shape(
            building_or_part,
            outer,
            building_or_part.footprint.is_circular,
            min_height,
            color,
        );

        for hole_index in 1..footprint.first_polygon_u().len() {
            let hole: &GroundPositions = &footprint.first_polygon_u()[hole_index];
            self.push_wall_shape(
                building_or_part,
                hole,
                footprint.is_circular,
                min_height,
                color,
            );
        }
        / ********/
    }

    fn push_wall_shape(
        &mut self,
        building_or_part: &mut BuildingOrPart,
        hole: &GroundPositions,
        is_circular: bool,
        min_height: f32,
        color: RenderColor,
    ) {
        // todo: thread 'main' panicked at src/to_3d.rs:544:51:   https://www.openstreetmap.org/way/313425087
        if hole.is_empty() {
            // empty by purpose. replaced by parts
            // println!("footprint.positions.is_empty {}", building_or_part.id);
            return;
        }
        let position = hole.last().unwrap();
        // todo: fn for next 3 lines
        let height = self.calc_roof_position_height(position, building_or_part);
        let mut last_gpu_position_down = position.to_gpu_position(min_height);
        let mut last_gpu_position_up = position.to_gpu_position(height);

        //for edge_index in 0..edges {

        // First vertex index of the comming walls
        let mut to_last_index = (hole.len() * 2 - 2) as isize;

        for position in hole.iter() {
            let height = self.calc_roof_position_height(position, building_or_part);
            let this_gpu_position_down = position.to_gpu_position(min_height);
            let this_gpu_position_up = position.to_gpu_position(height);

            // Walls
            if is_circular {
                self.push_square_soft(
                    to_last_index,
                    this_gpu_position_down,
                    this_gpu_position_up,
                    color,
                );
            } else {
                self.push_square(
                    last_gpu_position_down,
                    this_gpu_position_down,
                    last_gpu_position_up,
                    this_gpu_position_up,
                    color,
                );
            }
            to_last_index = -2;

            // Roof Points for triangulation and Onion, Positions for a Phyramide
            last_gpu_position_down = this_gpu_position_down;
            last_gpu_position_up = this_gpu_position_up;
        }
    }

    //// basic pushes: ////

    fn push_square_soft(
        &mut self,
        to_last_index: isize,
        down: RenderPosition,
        up: RenderPosition,
        color: RenderColor,
    ) {
        let start_index = self.attributes.vertices_positions.len();

        // Push the for colors
        self.attributes.vertices_colors.push(color);
        self.attributes.vertices_colors.push(color);
        // Push the two new positions
        self.attributes.vertices_positions.push(down);
        self.attributes.vertices_positions.push(up);

        // Push first and second treeangle
        // Calculate indexi of the square
        let index00 = start_index + O;
        let index10 = start_index + 1;
        let index01 = (index00 as isize + to_last_index) as usize;
        let index11 = (index10 as isize + to_last_index) as usize;

        //println!(
        //    "10: {index10} {edge_index} {ec} {ring_index} {}",
        //    (edge_index + 1) % ec,
        //);
        // Push indices of two treeangles
        self.push_3_indices([index00, index10, index01]);
        self.push_3_indices([index10, index11, index01]);
    }

    fn push_square(
        &mut self,
        down_left: RenderPosition,
        down_right: RenderPosition,
        up_left: RenderPosition,
        up_right: RenderPosition,
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
        self.push_3_indices([index /*....*/, index + 1, index + 2]);
        self.push_3_indices([index /*.*/+ 1, index + 3, index + 2]);
    }

    fn push_3_indices(&mut self, indexi: [usize; 3]) {
        // println!("i3: {:?}", indexi);
        self.attributes.indices_to_vertices.push(indexi[0] as u32);
        self.attributes.indices_to_vertices.push(indexi[1] as u32);
        self.attributes.indices_to_vertices.push(indexi[2] as u32);
    }
}

#[derive(Clone, Debug)]
struct ExtrudeRing {
    radius: f32,
    height: f32,
}

#[derive(Clone, Debug)]
struct Silhouette {
    ring_edges: Vec<ExtrudeRing>,
}

macro_rules! _er {
    (r,h) => (
        $crate::symbolic_3d::ExtrudeRing::new(
            radius: r,
            height: h,
        )

    );
}
