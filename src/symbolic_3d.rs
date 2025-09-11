// geo primitives
use geo::{Coord, LineString, Point, Rotate, TriangulateEarcut}; // Triangle
//use VecDeque::pop_front;

use crate::footprint::Footprint;
use crate::kernel_in::{BuildingOrPart, BuildingsAndParts, GroundPosition, RoofShape};
use crate::kernel_out::{OsmMeshAttributes, RenderColor, RenderPosition, RenderPositions};

///////////////////////////////////////////////////////////////////////////////////////////////////
// OSM ////////////////////////////////////////////////////////////////////////////////////////////

// Constants / Parameters
static MULTI_MESH: bool = false;
static _GPU_POSITION_NULL: RenderPosition = [0.0, 0.0, 0.0];
static O: usize = 0; // Just to silent lint, make some lines equal and to show, the Offset may also be 0

// Local methodes of GroundPosition, only to be used in the renderer!
pub fn to_gpu_position(coord: &Coord, height: f64) -> RenderPosition {
    [coord.x as f32, height as f32, -coord.y as f32] // -y bedause: OSM +nord => GPU -Z
}

impl Footprint {
    fn _get_gpu_positions(&self, height: f64) -> RenderPositions {
        let mut roof_gpu_positions: RenderPositions = Vec::new();
        for polygon in self.multipolygon.iter() {
            for position in polygon.exterior() {
                let this_gpu_position_up = to_gpu_position(position, height);
                roof_gpu_positions.push(this_gpu_position_up);
            }

            for hole in polygon.interiors() {
                // println!("    hole_index: {hole_index} len:{}", self.polygons[polygon_index].len());
                for position in hole {
                    let this_gpu_position_up = to_gpu_position(position, height);
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
        let _roof_height = building_or_part.roof_height;
        // println!("- m: {} w:{} r:{}", min_height, wall_height, roof_height);

        // https://docs.rs/geo/latest/geo/geometry/struct.LineString.html#impl-IsConvex-for-LineString%3CT%3E

        let color = building_or_part.building_color;
        let roof_color = building_or_part.roof_color;

        match building_or_part.roof_shape {
            //
            RoofShape::Skillion => {
                self.push_skillion(building_or_part, roof_color);
            }

            /**** /
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
            / **/
            _ => self.push_flat(&mut building_or_part.footprint, wall_height, roof_color),
        }

        self.push_walls(building_or_part, min_height, color);

        if min_height > 0.0 {
            self.push_flat(&mut building_or_part.footprint, -min_height, roof_color);
        }
    }

    fn prepare_roof(&mut self, _: &BuildingOrPart) {
        // println!("angle: {}", _building_part.roof_angle);
        // todo:
        // Add positions below roof first etc.
        // rotate a foodprint mirror
        // prepare height calculation
    }

    /**/
    fn calc_skillion_position_height(
        &mut self,
        position: &GroundPosition,
        building_or_part: &BuildingOrPart,
    ) -> f64 {
        // println!("ph: position: {:?}", position);
        let dist_to_center: Point = (*position - building_or_part.footprint.center).into();
        let position_rotated = dist_to_center.rotate_around_point(
            building_or_part.roof_angle.to_degrees(),
            building_or_part.footprint.center.into(),
        );
        let inclination =
            building_or_part.roof_height / building_or_part.bounding_box_rotated.height();
        let rotated_footprint_south = building_or_part.bounding_box_rotated.min().y;
        building_or_part.wall_height + building_or_part.roof_height
            - f64::abs(position_rotated.y() - rotated_footprint_south) * inclination
    }
    /** /

    fn calc_gabled_position_height(
        &mut self,
        position: &GroundPosition,
        building_or_part: &BuildingOrPart,
    ) -> FGP {
        let east = position
            .sub(building_or_part.footprint.center)
            .rotate(-building_or_part.roof_angle) // Rotate against the actual angle to got 0 degrees
            .y
            + building_or_part.footprint.shift;

        let width = building_or_part.bounding_box_rotated.north
            - building_or_part.bounding_box_rotated.south;
        let inclination = building_or_part.roof_height * 2. / width;

        building_or_part.wall_height + building_or_part.roof_height - FGP::abs(east) * inclination
    }
    / **/

    fn calc_roof_position_height(
        &mut self,
        position: &GroundPosition,
        building_or_part: &BuildingOrPart,
    ) -> f64 {
        match building_or_part.roof_shape {
            RoofShape::Skillion => self.calc_skillion_position_height(position, building_or_part),
            // RoofShape::Gabled => self.calc_gabled_position_height(position, building_or_part),
            _ => building_or_part.wall_height,
        }
    }

    // todo: use skilleon with height = constant
    fn push_flat(&mut self, footprint: &mut Footprint, height: f64, color: RenderColor) {
        // Why not do it with multipolygon:
        // A) the redundant, way ends get unused pushed indices
        // B) there is no earcut_triangles_raw for multipolygon
        for polygon in footprint.multipolygon.iter() {
            let roof_index_start = self.attributes.vertices_positions.len() as u32;
            // todo?: use get_triangulates?
            let mut triangles = polygon.earcut_triangles_raw();
            let vertices = triangles.vertices;
            //println!("ttt {roof_index_start} triangles: {:?}", triangles);

            const VALUES_PER_COORDINATE: usize = 2;
            const DROPP_LAST: usize = 1;
            let max = vertices.len() / VALUES_PER_COORDINATE - DROPP_LAST;

            for i in 0..max {
                let x = vertices[i * VALUES_PER_COORDINATE + O];
                let y = vertices[i * VALUES_PER_COORDINATE + 1];
                let gpu = [x as f32, height.abs() as f32, -y as f32]; // -y bedause: OSM +nord => GPU -Z
                self.attributes.vertices_positions.push(gpu);
                self.attributes.vertices_colors.push(color);
            }

            if height < 0.0 {
                triangles.triangle_indices.reverse();
            }

            for index in triangles.triangle_indices {
                self.attributes
                    .indices_to_vertices
                    .push(roof_index_start + index as u32);
            }
        }
    }

    fn push_skillion(&mut self, building_or_part: &mut BuildingOrPart, color: RenderColor) {
        /* */
        let footprint = &building_or_part.footprint;
        for (polygon_index, polygon) in footprint.multipolygon.iter().enumerate() {
            let roof_index_start = self.attributes.vertices_positions.len();
            // todo: first=last unused pushed: See push_flat?
            for coord in polygon.exterior() {
                let height = self.calc_roof_position_height(coord, building_or_part);
                self.attributes
                    .vertices_positions
                    .push(to_gpu_position(coord, height));
                self.attributes.vertices_colors.push(color);
            }

            let indices = footprint.get_triangulates(polygon_index);
            // println!("triangles: {:?}", &indices);

            for index in indices {
                self.attributes
                    .indices_to_vertices
                    .push((roof_index_start + index) as u32);
            }
        }
        /* */
    }

    /** /
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
        for _polygon_index in 0..building_or_part.footprint.multipolygon.len() {
            // todo: gabled for cutted buildings
        }

        let mut footprint = Footprint::new(); // &building_or_part.footprint;
        footprint.multipolygon[FIRST_POLYGON][OUTER_POLYGON] = side;

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

        for position in footprint.multipolygon[FIRST_POLYGON][OUTER_POLYGON].iter() {
            let height = self.calc_roof_position_height(position, building_or_part);
            self.attributes
                .vertices_positions
                .push(position.to_gpu_position(height))
        }

        for _position in &footprint.multipolygon[FIRST_POLYGON][OUTER_POLYGON] {
            self.attributes.vertices_colors.push(color);
        }
    }

    fn push_phyramid(
        &mut self,
        footprint: &Footprint,
        wall_height: FGP,
        roof_height: FGP,
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
        wall_height: FGP,
        roof_height: FGP,
        color: RenderColor,
    ) {
        let mut ring_edges: Vec<ExtrudeRing> = Vec::new();
        const STEPS: usize = 10;
        for step in 0..STEPS {
            let angle = f32::to_radians((step * STEPS) as f32);
            ring_edges.push(ExtrudeRing {
                radius: angle.cos() as FGP,
                height: angle.sin() as FGP,
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
        wall_height: FGP,
        roof_height: FGP,
        pike: GroundPosition,
    ) -> RenderPosition {
        let gpu_x: FGP = (edge.x - pike.x) * ring.radius + pike.x;
        let gpu_z: FGP = (edge.y - pike.y) * ring.radius + pike.y;
        let gpu_y: FGP = wall_height + roof_height * ring.height;
        [gpu_x as f32, gpu_y as f32, -gpu_z as f32] // Why -z?  Should be in an extra fn!
    }

    fn push_extrude(
        &mut self,
        footprint: &Footprint,
        silhouette: Silhouette,
        wall_height: FGP,
        roof_height: FGP,
        color: RenderColor,
    ) {
        let soft_edges = footprint.multipolygon[FIRST_POLYGON][OUTER_POLYGON].len() > 8;
        let mut gpu_positions: GpuPositions = Vec::new();
        for (ring_index, ring) in silhouette.ring_edges.iter().enumerate() {
            gpu_positions.push(Vec::new());
            for edge in footprint.multipolygon[FIRST_POLYGON][OUTER_POLYGON].iter() {
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
        let edges = footprint.multipolygon[FIRST_POLYGON][OUTER_POLYGON].len();
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
            let pike = gpu_positions[rings][OUTER_POLYGON];
            self.attributes.vertices_positions.push(pike);
            self.attributes.vertices_colors.push(color);
        }
        // println!("self.attributes: {:?}", self.attributes);
    }

    fn push_soft_edges(
        &mut self,
        gpu_positions: &[RenderPositions],
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
        gpu_positions: &[RenderPositions],
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
        wall_height: FGP,
        roof_height: FGP,
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
    / **/

    fn push_walls(
        &mut self,
        building_or_part: &mut BuildingOrPart,
        min_height: f64,
        color: RenderColor,
    ) {
        let footprint = &building_or_part.footprint.clone();
        for polygon in &footprint.multipolygon {
            // println!("=wall polygon: {:?}", polygon);
            let outer = polygon.exterior();
            self.push_wall_shape(
                building_or_part,
                outer,
                building_or_part.footprint.is_circular,
                min_height,
                color,
            );

            for hole in polygon.interiors() {
                self.push_wall_shape(
                    building_or_part,
                    hole,
                    footprint.is_circular,
                    min_height,
                    color,
                );
            }
        }
    }

    fn push_wall_shape(
        &mut self,
        building_or_part: &mut BuildingOrPart,
        wall: &LineString,
        is_circular: bool,
        min_height: f64,
        color: RenderColor,
    ) {
        // First vertex index of the comming walls
        let mut to_last_index = (wall.coords().count() * 2 - 2) as isize;
        let mut last_gpu_position_down: [f32; 3] = [0.; 3];
        let mut last_gpu_position_up: [f32; 3] = [0.; 3];

        for (index, position) in wall.coords().enumerate() {
            let height = self.calc_roof_position_height(position, building_or_part);
            let this_gpu_position_down = to_gpu_position(position, min_height);
            let this_gpu_position_up = to_gpu_position(position, height);

            if index > 0 {
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
            }

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
        self.push_3_indices([index + O, index + 1, index + 2]);
        self.push_3_indices([index + 1, index + 3, index + 2]);
    }

    fn push_3_indices(&mut self, indexi: [usize; 3]) {
        // println!("i3: {:?}", indexi);
        self.attributes.indices_to_vertices.push(indexi[0] as u32);
        self.attributes.indices_to_vertices.push(indexi[1] as u32);
        self.attributes.indices_to_vertices.push(indexi[2] as u32);
    }
}

#[derive(Clone, Debug)]
struct _ExtrudeRing {
    radius: f64,
    height: f64,
}

#[derive(Clone, Debug)]
struct _Silhouette {
    ring_edges: Vec<_ExtrudeRing>,
}
