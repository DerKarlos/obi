// OBI main

// use error_chain::error_chain;
use std::error::Error;

use reqwest;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;

use bevy::prelude::*;
use bevy::render::{
    mesh::Indices, //VertexAttributeValues},
    render_asset::RenderAssetUsages,
    render_resource::PrimitiveTopology,
};

//e triangulate::{self, formats, Polygon};
use csscolorparser::parse;
use triangulation::{Delaunay, Point};

// Constants / Parameters
const MULTI_MESH: bool = true;
const GPU_POS_NULL: [f32; 3] = [0.0, 0.0, 0.0];

// error_chain! {
//     foreign_links {
//         Io(std::io::Error);
//         HttpRequest(reqwest::Error);
//     }
// }

///////////////////////////////////////////////////////////////////////////////////////////////////
// JOSN ///////////////////////////////////////////////////////////////////////////////////////////

static NODE: &'static str = "node";
static WAY: &'static str = "way";
static _REL: &'static str = "rel";

static YES: &'static str = "yes";
static NO: &'static str = "no";

static DEFAULT_WALL_COLOR: &'static str = "grey";
static DEFAULT_ROOF_COLOR: &'static str = "red";

static DEFAULT_BUILDING_HEIGHT: f32 = 10.0;

static API_URL: &'static str = "https://api.openstreetmap.org/api/0.6/";

// todo: &str   https://users.rust-lang.org/t/requires-that-de-must-outlive-static-issue/91344/10
#[derive(Deserialize, Debug)]
struct JosnElement {
    #[serde(rename = "type")]
    element_type: String,
    id: u64,
    lat: Option<f64>,
    lon: Option<f64>,
    nodes: Option<Vec<u64>>,
    tags: Option<JosnTags>, // todo: use a map
}

#[derive(Deserialize, Debug, Clone, Default)]
struct JosnTags {
    // name: Option<String>,
    // building: Option<String>,
    #[serde(rename = "building:part")]
    building_part: Option<String>, // ??? &'static str>,
    #[serde(rename = "roof:shape")]
    roof_shape: Option<String>,
    #[serde(rename = "roof:colour")]
    roof_colour: Option<String>,
    colour: Option<String>,
    #[serde(rename = "roof:height")]
    roof_height: Option<String>,
    height: Option<String>,
    min_height: Option<String>,
}

#[derive(Deserialize, Debug)]
struct JsonData {
    elements: Vec<JosnElement>,
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// OSM ////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
struct GeographicCoordinates {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Clone, Copy, Debug)]
// todo: Nor?EastPos
struct GroundPosition {
    pub east: f32,
    pub north: f32,
}

impl fmt::Display for GroundPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.east, self.north)
    }
}

#[derive(Resource)]
struct CoordinatesAtGroundPositionNull {
    pub _pos: GeographicCoordinates,
}

struct OsmNode {
    pub position: GroundPosition,
}

fn parse_colour(colour: String) -> [f32; 4] {
    let colour_scc: csscolorparser::Color = parse(colour.as_str()).unwrap();
    // Bevy pbr color needs f32, The parse has no .to_f32_array???
    // https://docs.rs/csscolorparser/latest/csscolorparser/
    [
        colour_scc.r as f32,
        colour_scc.g as f32,
        colour_scc.b as f32,
        colour_scc.a as f32,
    ]
}

fn coordinates_of_way(way_id: u64) -> GeographicCoordinates {
    // DONT USE:   https://api.openstreetmap.org/api/0.6/way/121486088/full.json
    // https://master.apis.dev.openstreetmap.org/api/0.6/way/121486088/full.json
    // The test-server does not have needed objects (like Reifenberg), but they could be PUT into

    let url = format!("{}way/{}/full.json", API_URL, way_id);

    // Get OSM data from API and convert Json to Rust types. See https://serde.rs
    let json_way: JsonData = reqwest::blocking::get(url).unwrap().json().unwrap();

    let mut latitude: f64 = 0.0;
    let mut longitude: f64 = 0.0;
    let mut nodes_divider: f64 = -1.;

    // add the coordinates of all nodes
    for element in json_way.elements {
        if element.element_type == NODE {
            if nodes_divider >= 0. {
                latitude += element.lat.unwrap();
                longitude += element.lon.unwrap();
            }
            nodes_divider += 1.0;
        }
    }
    // calculate and return everedge
    latitude /= nodes_divider;
    longitude /= nodes_divider;
    GeographicCoordinates {
        latitude,
        longitude,
    }
}

fn scan_json(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    coordinates_at_ground_position_null: GeographicCoordinates,
) {
    // https://wiki.openstreetmap.org/wiki/API_v0.6#Retrieving_map_data_by_bounding_box:_GET_/api/0.6/map
    let range = 15.0 / LAT_FAKT; // First test with 15 meter
    let left = coordinates_at_ground_position_null.longitude - range;
    let right = coordinates_at_ground_position_null.longitude + range;
    let top = coordinates_at_ground_position_null.latitude + range;
    let bottom = coordinates_at_ground_position_null.latitude - range;
    // let left_top = to_position(&CoordinatesAtGroundPositionNull, left, top);
    // println!("range: left_top={} url={}", left_top, url);
    // GET   /api/0.6/map?bbox=left,bottom,right,top
    let url = format!(
        "{}map.json?bbox={},{},{},{}",
        API_URL, left, bottom, right, top
    );
    // range: x=4209900.5 z=-4290712 url=
    // https://api.openstreetmap.org/api/0.6/map.json?bbox=11.135635953165316,49.75577293983198,11.135905980168015,49.75604296683468
    // https://api.openstreetmap.org/api/0.6/map.json?bbox=76.36808519471933,64.41713173392363,76.75875957883649,64.50167155517451

    //t url = format!("https://api.openstreetmap.org/api/0.6/way/{}/full.json", way_id);
    let json_map: JsonData = reqwest::blocking::get(url).unwrap().json().unwrap();

    //let nodes_divider = Map(id:u64, node:OsmNode);
    let mut nodes_map = HashMap::new();

    let mut cmesh = CMesh::new();

    for element in json_map.elements {
        if element.element_type == NODE {
            let osm_node = OsmNode {
                position: to_position(
                    &coordinates_at_ground_position_null,
                    element.lat.unwrap(),
                    element.lon.unwrap(),
                ),
            };
            nodes_map.insert(element.id, osm_node);
            // println!("Node: id = {:?} lat = {:?} lon = {:?}", element.id, element.lat.unwrap(), element.lon.unwrap() );
        }

        // todo: string!("way") {
        if element.element_type == WAY {
            // println!("element = {:?}", element);
            let tags = element.tags.unwrap(); // JosnTags { ..default() }; //ttt
            let building_part = tags.building_part.unwrap_or(NO.to_string());
            // let name = tags.name.unwrap_or("-/-".to_string());
            // println!(" Way: building = {:?}  name = {:?}" name,);
            if building_part != YES {
                continue;
            };

            // Colors and Materials
            let colour: [f32; 4] =
                parse_colour(tags.colour.unwrap_or(DEFAULT_WALL_COLOR.to_string()));
            let roof_colour =
                parse_colour(tags.roof_colour.unwrap_or(DEFAULT_ROOF_COLOR.to_string()));
            // println!("colors: {:?} {:?}", colour, roof_colour);

            // Height
            let mut min_height = 0.0;
            let mut part_height = DEFAULT_BUILDING_HEIGHT;
            let mut roof_height = 0.0;
            if let Some(height) = tags.min_height {
                min_height = height.parse().unwrap();
            }
            if let Some(height) = tags.height {
                part_height = height.parse().unwrap();
            }
            if let Some(height) = tags.roof_height {
                roof_height = height.parse().unwrap();
                part_height -= roof_height;
            }

            // Get building walls from nodes
            let nodes = element.nodes.unwrap();
            let mut last_pos_down = GPU_POS_NULL;
            let mut last_pos_up = GPU_POS_NULL;

            // roof
            let roof_shape = tags.roof_shape.unwrap_or("flat".to_string());
            let mut roof_polygon: Vec<Point> = vec![];
            let mut roof_positions: Vec<Position> = vec![];
            let mut sum_east = 0.0;
            let mut sum_north = 0.0;
            println!("roof_shape: {}", roof_shape);

            // https://docs.rs/geo/latest/geo/geometry/struct.LineString.html#impl-IsConvex-for-LineString%3CT%3E
            for (index, node_id) in nodes.iter().rev().enumerate() {
                let node = nodes_map.get(&node_id).unwrap();

                let this_pos_down = [node.position.east, min_height, node.position.north];
                let this_pos_up = [node.position.east, part_height, node.position.north];
                let roof_point = Point::new(node.position.east, node.position.north);

                if index > 0 {
                    // skip first node = last
                    cmesh.push_square(
                        last_pos_down,
                        this_pos_down,
                        last_pos_up,
                        this_pos_up,
                        colour,
                    );
                    // The polygon node list is "closed": Last is connected to first
                    roof_polygon.push(roof_point);
                    roof_positions.push(this_pos_up);
                    sum_east += node.position.east;
                    sum_north += node.position.north;
                }
                last_pos_down = this_pos_down;
                last_pos_up = this_pos_up;
            }
            // center of way
            const LAST_AS_IT_IS_EQUAL_TO_FIRST: usize = 1;
            let count = (nodes.len() - LAST_AS_IT_IS_EQUAL_TO_FIRST) as f32;
            sum_east /= count;
            sum_north /= count;

            if roof_shape == "pyramidal".to_string() {
                cmesh.push_pyramid(
                    roof_positions,
                    [sum_east, part_height + roof_height, sum_north],
                    roof_colour,
                );
            } else if roof_shape == "onion".to_string() {
                cmesh.push_onion(
                    part_height,
                    roof_height,
                    roof_polygon,
                    Point {
                        x: sum_east,
                        y: sum_north,
                    },
                    roof_colour,
                );
            } else if roof_shape == "hipped".to_string() {
                cmesh.push_flat(roof_positions, roof_polygon, roof_colour);
            } else if roof_shape == "gabled".to_string() {
                cmesh.push_flat(roof_positions, roof_polygon, roof_colour);
            } else if roof_shape == "flat".to_string() {
                cmesh.push_flat(roof_positions, roof_polygon, roof_colour);
            }

            if MULTI_MESH {
                //println!("MULTI_MESH");
                cmesh.spawn(commands, &mut meshes, &mut materials);
                cmesh = CMesh::new();
            }
        }
    }

    if !MULTI_MESH {
        cmesh.spawn(commands, &mut meshes, &mut materials)
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// BEVY ///////////////////////////////////////////////////////////////////////////////////////////

// Define a "marker" component to mark the custom mesh. Marker components are often used in Bevy for
// filtering entities in queries with With, they're usually not queried directly since they don't contain information within them.
#[derive(Component)]
struct Controled;

fn setup(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    // Testing with this moderate complex building
    // https://www.openstreetmap.org/way/121486088#map=19/49.75594/11.13575&layers=D
    let reifenberg_id = 121486088;

    let coordinates_at_ground_position_null = if false {
        // Todo: remove test
        coordinates_of_way(reifenberg_id)
    } else {
        GeographicCoordinates {
            latitude: 49.755907953,
            longitude: 11.135770967,
        }
    };
    println!(
        "coordinates_of_way: {:?}",
        coordinates_at_ground_position_null
    );
    commands.insert_resource(CoordinatesAtGroundPositionNull {
        _pos: coordinates_at_ground_position_null.clone(),
    });

    // Dodo: get building ... other way
    scan_json(
        &mut commands,
        meshes,
        materials,
        coordinates_at_ground_position_null,
    );

    // Transform for the camera and lighting, looking at (0,0,0) (the position of the mesh).
    let camera_and_light_transform =
        Transform::from_xyz(30., 20., 30.).looking_at(Vec3::new(0., 10., 0.), Vec3::Y);

    // Camera in 3D space.
    commands.spawn(Camera3dBundle {
        transform: camera_and_light_transform,
        ..default()
    });

    // Light up the scene.
    //commands.spawn(PointLightBundle {
    //    transform: camera_and_light_transform,
    //    ..default()
    //});

    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(400., 500., 400.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

// System to receive input from the user,
// check out examples/input/ for more examples about user input.
fn input_handler(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Controled>>,
    time: Res<Time>,
) {
    if keyboard_input.pressed(KeyCode::KeyX) {
        for mut transform in &mut query {
            transform.rotate_x(time.delta_seconds() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::KeyY) {
        // German: Z
        for mut transform in &mut query {
            transform.rotate_y(time.delta_seconds() / 1.2);
            //println!("Key y");
        }
    }
    if keyboard_input.pressed(KeyCode::KeyU) {
        // German: Z
        for mut transform in &mut query {
            transform.rotate_y(-time.delta_seconds() / 1.2);
            //println!("Key y");
        }
    }
    if keyboard_input.pressed(KeyCode::KeyZ) {
        // German: Y
        for mut transform in &mut query {
            transform.rotate_z(time.delta_seconds() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::KeyR) {
        for mut transform in &mut query {
            transform.look_to(Vec3::NEG_Z, Vec3::Y);
        }
    }
}

// ??? //////////////////////

type ColorAlpha = [f32; 4];
type Position = [f32; 3];
//type XzVec = [f32;2];

/** Factor to calculate meters from gps coordiantes.decimals (latitude, Nort/South position) */
static LAT_FAKT: f64 = 111100.0; // 111285; // exactly enough  111120 = 1.852 * 1000.0 * 60  // 1 NM je Bogenminute: 1 Grad Lat = 60 NM = 111 km, 0.001 Grad = 111 m
static PI: f32 = std::f32::consts::PI;

fn to_position(coordiantes: &GeographicCoordinates, lat: f64, lon: f64) -> GroundPosition {
    // the closer to the pole, the smaller the tiles size in meters get
    let lon_fakt = LAT_FAKT * ((lat / 180. * PI as f64).abs()).cos(); // Longitude(Längengrad) West/East factor
                                                                      // actual coor - other coor = relative grad/meter ground position
    let east = ((lon - coordiantes.longitude) * lon_fakt) as f32;
    let north = ((lat - coordiantes.latitude) * LAT_FAKT) as f32;
    /*return*/
    GroundPosition { east, north }
}

// "CLASS" Custom Mesh /////////////////////////////////////////

struct CMesh {
    colors: Vec<ColorAlpha>,          // format: Float32x4
    position_vertices: Vec<Position>, // 3 coordinates * x Positions. The corners are NOT reused to get hard Kanten
    indices: Vec<u32>,
}

impl CMesh {
    pub fn new() -> Self {
        Self {
            colors: vec![],
            position_vertices: vec![],
            indices: vec![],
        }
    }

    pub fn spawn(
        &mut self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) {
        let mesh_handle = meshes.add(self.get_mesh());
        commands.spawn((
            PbrBundle {
                mesh: mesh_handle,
                material: materials.add(Color::srgb(1.0, 1.0, 1.0)),
                ..default()
            },
            Controled,
        ));
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
        let _roof_rel = roof_height / 3.5; // relation of height and with of the standard "onion" shape

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
    }

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
        // First index of the comming 4 positions
        let index = self.position_vertices.len();
        // Push the for colors
        self.colors.push(colour);
        self.colors.push(colour);
        self.colors.push(colour);
        self.colors.push(colour);
        // Push the for positions
        self.position_vertices.push(down_left); // +0     2---3
        self.position_vertices.push(down_right); // +1     |   |
        self.position_vertices.push(up_left); // +2     0---1
        self.position_vertices.push(up_right); // +3
                                               // First treeangle
        self.indices.push((index + 0) as u32); //     2
        self.indices.push((index + 1) as u32); //     | \
        self.indices.push((index + 2) as u32); //     0---1
                                               // Secound treeangle
        self.indices.push((index + 1) as u32); //     2---3
        self.indices.push((index + 3) as u32); //       \ |
        self.indices.push((index + 2) as u32); //         1
    }

    pub fn get_mesh(&self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.position_vertices.clone())
        .with_inserted_indices(Indices::U32(self.indices.clone()))
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// MAIN ///////////////////////////////////////////////////////////////////////////////////////////

// https://github.com/DerKarlos/obi/tree/master/src

fn main() -> Result<(), Box<dyn Error>> {
    //fn main() -> Result<()> {
    println!("*********  Hi, I'm OBI, the OSM Buiding Inspector  *********");

    // BEVY-App
    App::new()
        .add_plugins(DefaultPlugins)
        // set the global default clear colour
        .insert_resource(ClearColor(Color::srgb(0.0, 0.5, 0.0)))
        .add_systems(Startup, setup)
        .add_systems(Update, input_handler)
        .run();

    Ok(())
}
