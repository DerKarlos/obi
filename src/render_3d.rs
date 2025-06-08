use crate::kernel_in::{BuildingPart, GroundPosition, RoofShape};
use crate::kernel_out::{GpuPosition, OsmMeshAttributes, RenderColor};
use crate::shape::Shape;
use std::cmp::min;
use std::ops::Sub;

///////////////////////////////////////////////////////////////////////////////////////////////////
// OSM ////////////////////////////////////////////////////////////////////////////////////////////

// Constants / Parameters
static MULTI_MESH: bool = false;
static _GPU_POSITION_NULL: GpuPosition = [0.0, 0.0, 0.0];

// Local methodes of GroundPosition, only to be used in the renderer!
impl GroundPosition {
    fn to_gpu_position(self, height: f32) -> GpuPosition {
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
        // println!("- m: {} w:{} r:{}", min_height, wall_height, roof_height);

        // https://docs.rs/geo/latest/geo/geometry/struct.LineString.html#impl-IsConvex-for-LineString%3CT%3E

        let color = building_part.building_color;
        let roof_color = building_part.roof_color;

        match building_part.roof_shape {
            //
            RoofShape::Skillion => {
                self.push_skillion(building_part, roof_color);
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

            RoofShape::Dome => self.push_dome(
                &building_part.footprint,
                wall_height,
                roof_height,
                roof_color,
            ),

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
        println!("roof_angle: {}", building_part.roof_angle.to_degrees());
        let inclination = building_part.roof_height
            / (building_part.bounding_box_rotated.east - building_part.bounding_box_rotated.west); // Höhen/Tiefe der Nodes/Ecken berechenen

        building_part.wall_height + building_part.roof_height
            - f32::abs(east - building_part.bounding_box_rotated.west) * inclination
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
            "gabled - East: {east} width: {width} roof_height: {rh} width: {width} Inc: {inclination} height: {height}"
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

        // ? why .rev()?  see negativ ???
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
            let height = self.calc_roof_position_height(position, building_part);
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

        self.push_roof_shape(face1, color, building_part);
        self.push_roof_shape(face2, color, building_part);
    }

    fn push_roof_shape(
        &mut self,
        side: Vec<GroundPosition>,
        color: RenderColor,
        building_part: &BuildingPart,
    ) {
        let mut footprint = Shape::new(); // &building_part.footprint;
        footprint.positions = side;

        let roof_index_offset = self.attributes.vertices_positions.len();
        let indices = footprint.get_triangulate_indices();
        //println!("offset: {roof_index_offset}  indices: {:?}", &indices);

        // ? why .rev() ?  see negativ ???
        for index in indices.iter().rev() {
            self.attributes
                .indices_to_vertices
                .push((roof_index_offset + index) as u32);
        }

        for position in footprint.positions.iter() {
            let height = self.calc_roof_position_height(position, building_part);
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

    fn push_phyramid(
        &mut self,
        footprint: &Shape,
        wall_height: f32,
        roof_height: f32,
        color: RenderColor,
    ) {
        let mut ring_edges: Vec<ExtrudeRing> = Vec::new();
        ring_edges.push(ExtrudeRing {
            radius: 1.,
            height: 0.,
        });
        ring_edges.push(ExtrudeRing {
            radius: 0.,
            height: 1.,
        });
        let silhouette = Silhouette { ring_edges };
        self.push_extrude(footprint, silhouette, wall_height, roof_height, color);
    }

    fn push_dome(
        &mut self,
        footprint: &Shape,
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
    ) -> GpuPosition {
        let gpu_x = (edge.east - pike.east) * ring.radius + pike.east;
        let gpu_z = (edge.north - pike.north) * ring.radius + pike.north;
        let gpu_y = wall_height + roof_height * ring.height;
        [gpu_x, gpu_y, -gpu_z] // Why -z?  Should be in an extra fn!
    }

    fn push_extrude(
        &mut self,
        footprint: &Shape,
        silhouette: Silhouette,
        wall_height: f32,
        roof_height: f32,
        color: RenderColor,
    ) {
        let soft_edges = footprint.positions.len() > 0; //ttt 8;
        let mut gpu_positions: Vec<Vec<GpuPosition>> = Vec::new();
        for (ring_index, ring) in silhouette.ring_edges.iter().enumerate() {
            gpu_positions.push(Vec::new());
            for edge in footprint.positions.iter() {
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
        let edges = footprint.positions.len();
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
        gpu_positions: &Vec<Vec<GpuPosition>>,
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
        let index00 = (edge_index + 0) % ec + (ring_index + 0) * ec;
        let index10 = (edge_index + 1) % ec + (ring_index + 0) * ec;
        let index01 = (edge_index + 0) % ec + (ring_index + 1) * ec;
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
        gpu_positions: &Vec<Vec<GpuPosition>>,
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
        footprint: &Shape,
        wall_height: f32,
        roof_height: f32,
        color: RenderColor,
    ) {
        let mut ring_edges: Vec<ExtrudeRing> = Vec::new();
        ring_edges.push(ExtrudeRing {
            radius: 1.00,
            height: 0.00,
        });
        ring_edges.push(ExtrudeRing {
            radius: 1.12,
            height: 0.09,
        });
        ring_edges.push(ExtrudeRing {
            radius: 1.27,
            height: 0.15,
        });
        ring_edges.push(ExtrudeRing {
            radius: 1.36,
            height: 0.27,
        });
        ring_edges.push(ExtrudeRing {
            radius: 1.28,
            height: 0.42,
        });
        ring_edges.push(ExtrudeRing {
            radius: 1.10,
            height: 0.51,
        });
        ring_edges.push(ExtrudeRing {
            radius: 0.95,
            height: 0.53,
        });
        ring_edges.push(ExtrudeRing {
            radius: 0.62,
            height: 0.58,
        });
        ring_edges.push(ExtrudeRing {
            radius: 0.49,
            height: 0.61,
        });
        ring_edges.push(ExtrudeRing {
            radius: 0.21,
            height: 0.69,
        });
        ring_edges.push(ExtrudeRing {
            radius: 0.10,
            height: 0.79,
        });
        ring_edges.push(ExtrudeRing {
            radius: 0.00,
            height: 1.00,
        });

        let silhouette = Silhouette { ring_edges };
        self.push_extrude(footprint, silhouette, wall_height, roof_height, color);
    }

    fn push_walls(
        &mut self,
        building_part: &mut BuildingPart,
        min_height: f32,
        color: RenderColor,
    ) {
        let position = building_part.footprint.positions.last().unwrap();
        // todo: fn for next 3 lines
        let height = self.calc_roof_position_height(position, building_part);
        let mut last_gpu_position_down = position.to_gpu_position(min_height);
        let mut last_gpu_position_up = position.to_gpu_position(height);

        for position in building_part.footprint.positions.iter() {
            let height = self.calc_roof_position_height(position, building_part);
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
