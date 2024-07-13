use error_chain::error_chain;
use reqwest;
use serde::Deserialize;
use std::collections::HashMap;

use bevy::prelude::*;
use bevy::render::{
    mesh::Indices, //VertexAttributeValues},
    render_asset::RenderAssetUsages,
    render_resource::PrimitiveTopology,
};

//e triangulate::{self, formats, Polygon};
use triangulation::{Delaunay, Point};
use csscolorparser::parse;

const MULTI_MESH: bool = true;
const POS0:[f32; 3] = [0.0,0.0,0.0];

// Define a "marker" component to mark the custom mesh. Marker components are often used in Bevy for
// filtering entities in queries with With, they're usually not queried directly since they don't contain information within them.
#[derive(Component)]
struct Controled;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}

// JOSN /////////////////////////////////

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

#[derive(Deserialize, Debug)]
struct JosnTags {
    // name: Option<String>,
    // building: Option<String>,
    #[serde(rename = "building:part")]
    building_part: Option<String>,
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

// OSM ///////////////////////

#[derive(Clone, Copy, Debug)]
struct GeoPos {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Clone, Copy, Debug)]
struct XzPos {
    pub x: f32,
    pub z: f32,
}


#[derive(Resource)]
struct GeoPosAtGPU0 {
    pub _pos: GeoPos,
}

struct OsmNode {
    pub pos: XzPos,
}


fn geopos_of_way (way_id: u64) -> GeoPos {

    // DONT USE:   https://api.openstreetmap.org/api/0.6/way/121486088/full.json
    // https://master.apis.dev.openstreetmap.org/api/0.6/way/121486088/full.json
    // The test-server does not have needed objects (like Reifenberg), but they could be PUT into

    let url = format!("https://api.openstreetmap.org/api/0.6/way/{}/full.json", way_id);

    // Get OSM data from API and convert Json to Rust types. See https://serde.rs
    let json: JsonData = reqwest::blocking::get(url).unwrap().json().unwrap();

    let mut lat: f64 = 0.0;
    let mut lon: f64 = 0.0;
    let mut nodes: f64 = 0.;

    // add the coordinates of oll nodes
    for element in json.elements {
        if element.element_type == "node".to_string() {
            if nodes > 0. {
                lat += element.lat.unwrap();
                lon += element.lon.unwrap();
            }
            nodes += 1.0;
        }

    }
    // calculate and return everedge
    nodes -= 1.0;
    lat /= nodes;
    lon /= nodes;
    println!("geopos_of_way: lat = {:?} lon = {:?}", lat, lon );
    GeoPos{lat,lon}

}

fn scan_json(commands: &mut Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, geo_pos_at_gpu0: GeoPos) {

    // https://wiki.openstreetmap.org/wiki/API_v0.6#Retrieving_map_data_by_bounding_box:_GET_/api/0.6/map
    let range = 15.0 / LAT_FAKT; // First test with 15 meter
    let left   = geo_pos_at_gpu0.lon-range;
    let right  = geo_pos_at_gpu0.lon+range;
    let top    = geo_pos_at_gpu0.lat+range;
    let bottom = geo_pos_at_gpu0.lat-range;
    let test = to_position(&geo_pos_at_gpu0, left, top);
    // GET                                          /api/0.6/map?bbox=left,bottom,right,top
    let url = format!("https://api.openstreetmap.org/api/0.6/map.json?bbox={},{},{},{}", left,bottom,right,top);
    println!("range: x={} z={} url={}", test.x,test.z,url);
    // range: x=4209900.5 z=-4290712 url=
    // https://api.openstreetmap.org/api/0.6/map.json?bbox=11.135635953165316,49.75577293983198,11.135905980168015,49.75604296683468
    // https://api.openstreetmap.org/api/0.6/map.json?bbox=76.36808519471933,64.41713173392363,76.75875957883649,64.50167155517451

    //t url = format!("https://api.openstreetmap.org/api/0.6/way/{}/full.json", way_id);
    let json: JsonData = reqwest::blocking::get(url).unwrap().json().unwrap();

    //let nodes = Map(id:u64, node:OsmNode);
    let mut nodes_map = HashMap::new();

    let mut cmesh = CMesh::new();

    for element in json.elements {
        if element.element_type == "node".to_string() {
            let osm_node = OsmNode{pos: to_position( &geo_pos_at_gpu0, element.lat.unwrap(), element.lon.unwrap())};
            nodes_map.insert(element.id, osm_node);
            // println!("Node: id = {:?} lat = {:?} lon = {:?}", element.id, element.lat.unwrap(), element.lon.unwrap() );
        }
        if element.element_type == "way".to_string() {
            let tags = element.tags.unwrap();
            let building_part = tags.building_part.unwrap_or("-/-".to_string());
            // let name = tags.name.unwrap_or("-/-".to_string());
            // println!(" Way: building = {:?}  name = {:?}" name,);
            if building_part != "yes" {continue;};

            // Colors and Materials
            let colour = tags.colour.unwrap_or("grey^".to_string());
            let colour = parse(colour.as_str()).unwrap();
            let colour = [ colour.r as f32, colour.g as f32, colour.b as f32, 1.0 ];

            let roof_colour = tags.roof_colour.unwrap_or("red".to_string());
            let roof_colour = parse(roof_colour.as_str()).unwrap();
            let roof_colour = [
                roof_colour.r as f32,
                roof_colour.g as f32,
                roof_colour.b as f32, 1.0
                ];
            println!("colors: {:?} {:?}",colour,roof_colour);

            // Height
            let mut min_height  = 0.0;
            let mut part_height = 10.0;
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
            let mut last_pos_down = POS0;
            let mut last_pos_up   = POS0;

            // roof
            let roof_shape = tags.roof_shape.unwrap_or("flat".to_string());
            let mut roof_polygon: Vec<Point> = vec![];
            let mut roof_positions: Vec<Position> = vec![];
            let mut x = 0.0;
            let mut z = 0.0;

            // https://docs.rs/geo/latest/geo/geometry/struct.LineString.html#impl-IsConvex-for-LineString%3CT%3E
            for (index,node_id) in nodes.iter().rev().enumerate() {
                let node = nodes_map.get(&node_id).unwrap();

                let this_pos_down = [node.pos.x, min_height,node.pos.z];
                let this_pos_up   = [node.pos.x,part_height,node.pos.z];
                let roof_point    = Point::new(node.pos.x,node.pos.z);

                if index>0 { // skip first node = last
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
                    x += node.pos.x;
                    z += node.pos.z;
                }
                last_pos_down = this_pos_down;
                last_pos_up   = this_pos_up;
            }
            // center of way
            let count = (nodes.len()-1) as f32;
            x /= count;
            z /= count;

            if roof_shape == "pyramidal".to_string() { cmesh.push_pyramid(roof_positions,[x,part_height+roof_height,z,],roof_colour); } else
            if roof_shape ==     "onion".to_string() { cmesh.push_onion(  part_height, roof_height, roof_polygon, Point{x,y:z}, roof_colour);      } else
            if roof_shape ==    "hipped".to_string() { cmesh.push_flat(   roof_positions, roof_polygon, roof_colour);                       } else
            if roof_shape ==    "gabled".to_string() { cmesh.push_flat(   roof_positions, roof_polygon, roof_colour);                       } else
            if roof_shape ==      "flat".to_string() { cmesh.push_flat(   roof_positions, roof_polygon, roof_colour);                       }

            if MULTI_MESH {
                //println!("MULTI_MESH");
                cmesh.spawn( commands, &mut meshes, &mut materials);
                cmesh = CMesh::new();

            }

        }
    }

    if !MULTI_MESH { cmesh.spawn( commands, &mut meshes, &mut materials) }

}


// MAIN ///////////////////

// https://github.com/DerKarlos/obi/tree/master/src

fn main() -> Result<()>
{     
    println!("Hi, I'm OBI, the OSM Buiding Inspector");

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

// BEVY ////////////////////

fn setup(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {

    // Testing with this moderate complex building
    // https://www.openstreetmap.org/way/121486088#map=19/49.75594/11.13575&layers=D
    let reifenberg_id = 121486088;

    // get geo-position at the 0 position of the GPU
    let geo_pos_at_gpu0 = if false { // Todo: remove test
        geopos_of_way(reifenberg_id)
    } else {
        GeoPos{lat: 49.755907953, lon: 11.135770967}
    };
    commands.insert_resource(GeoPosAtGPU0{_pos: geo_pos_at_gpu0.clone()});

    // Dodo: get building ... other way
    scan_json(&mut commands, meshes, materials, geo_pos_at_gpu0);

   // Transform for the camera and lighting, looking at (0,0,0) (the position of the mesh).
   let camera_and_light_transform =
        Transform::from_xyz(40., 40., 40.).looking_at(Vec3::ZERO, Vec3::Y);

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
        for mut transform in &mut query {
            transform.rotate_y(time.delta_seconds() / 1.2);
            //println!("Key y");
        }
    }
    if keyboard_input.pressed(KeyCode::KeyZ) {
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

type ColorAlpha = [f32;4];
type Position = [f32;3];
//type XzVec = [f32;2];

/** Factor to calculate meters from gps geo.decimals (latitude, Nort/South position) */
static LAT_FAKT: f64 = 111100.0; // 111285; // exactly enough  111120 = 1.852 * 1000.0 * 60  // 1 NM je Bogenminute: 1 Grad Lat = 60 NM = 111 km, 0.001 Grad = 111 m
static PI: f32 = std::f32::consts::PI;

fn to_position( geo_pos: &GeoPos, lat: f64, lon: f64) -> XzPos {
    // the closer to the pole, the smaller the tiles size in meters get
    let lon_fakt = LAT_FAKT * ((lat / 180. * PI as f64).abs()).cos(); // Longitude(Längengrad) West/East factor
    // actual geo pos - other geo pos = relative geo/meter scene pos
    let x = (lon - geo_pos.lon) * lon_fakt;
    let z = (lat - geo_pos.lat) * LAT_FAKT;
    /*return*/ XzPos{x: x as f32, z: z as f32}
}


// "CLASS" Custom Mesh /////////////////////////////////////////

struct CMesh {
    colors: Vec<ColorAlpha>, // format: Float32x4
    position_vertices: Vec<Position>, // 3 coordinates * x Positions. The corners are NOT reused to get hard Kanten
    indices: Vec<u32>,
}


impl CMesh {

    pub fn new() -> Self {
        Self{
            colors: vec![],
            position_vertices: vec![],
            indices: vec![],
        }
    }

    pub fn spawn(&mut self, commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>> ) {

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

    pub fn push_onion(&mut self, part_height: f32, roof_height: f32, roof_polygon: Vec<Point>, pike: Point, colour: [f32; 4]) {

        let shape_curve = 
        [   // -x-     |y|   from OSMgo, taken from picture pixle coordinates
            [0.0606,0.0054],
            [0.1202,0.0094],
            [0.2248,0.0510],
            [0.3096,0.0981],
            [0.3807,0.1571],
            [0.4351,0.2207],
            [0.4759,0.2992],
            [0.4947,0.3754],
            [0.5000,0.4454],
            [0.4937,0.5231],
            [0.4769,0.5875],
            [0.4330,0.6857],
            [0.3817,0.7604],
            [0.3263,0.8232],
            [0.2709,0.8727],
            [0.2092,0.9190],
            [0.1527,0.9544],
            [0.0847,0.9866],
            [0.0428,0.9976],
            [0.0000,1.0000],
        ];

        let columns = roof_polygon.len() as i32;
        let to_next_column = columns * 2;
        let roof_rel = roof_height / 3.5; // relation of height and with of the standard "onion" shape

        for point in shape_curve { // process all rings

            let curve_radius = point[0] as f32;
            let curve_up     = point[1] as f32;
            //println!("scale {} {} {} {}",curve_up,curve_radius, to_next_column, roof_height);

            let column_point = roof_polygon.last().unwrap();
            let pos_x = (column_point.x - pike.x) * curve_radius * roof_rel + pike.x;
            let pos_z = (column_point.y - pike.y) * curve_radius * roof_rel + pike.y;
            let mut last_pos = [ pos_x, part_height+roof_height*curve_up, pos_z ];

            for column_point in roof_polygon.iter() { // process one ring

                // push colors
                self.colors.push(colour);
                self.colors.push(colour);

                // push vertices
                let pos_x = (column_point.x - pike.x) * curve_radius * roof_rel + pike.x;
                let pos_z = (column_point.y - pike.y) * curve_radius * roof_rel + pike.y;
                let this_pos = [ pos_x, part_height+roof_height*curve_up, pos_z ];
                //println!("pso x z {} {} {:?} {:?}",pos_x,pos_z,last_pos,this_pos);
                let index = self.position_vertices.len() as i32;
                self.position_vertices.push(last_pos); // right vertice different than left to get corneres
                self.position_vertices.push(this_pos); // left - up=down to not get corners
                last_pos = this_pos;

                if curve_radius > 0. { // not if it is the last point/ring of the curve
                    // Push indices, first treeangle
                    self.indices.push( index                   as u32); // 0     2
                    self.indices.push((index+1               ) as u32); // 1     | \
                    self.indices.push((index+to_next_column  ) as u32); // 2     0---1
                    // Secound treeangle
                    self.indices.push((index+1               ) as u32); // 0     2---1
                    self.indices.push((index+to_next_column+1) as u32); // 1       \ |
                    self.indices.push((index+to_next_column  ) as u32); // 2         0
                    //println!("index {} {}",index,index+to_next_column);
                }

            } // ring

        } // all rings

    }

    pub fn push_flat(&mut self, positions: Vec<Position>, roof_polygon: Vec<Point>, colour: [f32; 4] ) {

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
            let mut index2 = index+1;
            if index2 >= positions.len() {index2 = 0}
            self.indices.push((roof_index_offset + index1) as u32);
            self.indices.push((roof_index_offset + index2) as u32);
            self.indices.push((roof_index_offset + pike_index_offset) as u32);
        }
        self.position_vertices.push(pike);
        self.colors.push(colour);
        //println!("ttt rio={} pio={} len={}",roof_index_offset, pike_index_offset,self.position_vertices.len() );
    }

    pub fn push_square(&mut self, down_left: Position, down_right: Position, up_left: Position, up_right: Position, colour: [f32; 4]  ) {
        // First index of the comming 4 positions
        let index = self.position_vertices.len();
        // Push the for colors
        self.colors.push(colour);
        self.colors.push(colour);
        self.colors.push(colour);
        self.colors.push(colour);
        // Push the for positions
        self.position_vertices.push( down_left);  // +0     2---3
        self.position_vertices.push( down_right); // +1     |   |
        self.position_vertices.push( up_left);    // +2     0---1
        self.position_vertices.push( up_right);   // +3
        // First treeangle
        self.indices.push((index+0) as u32); //     2
        self.indices.push((index+1) as u32); //     | \
        self.indices.push((index+2) as u32); //     0---1
        // Secound treeangle
        self.indices.push((index+1) as u32); //     2---3
        self.indices.push((index+3) as u32); //       \ |
        self.indices.push((index+2) as u32); //         1
    }

    pub fn get_mesh(&self) -> Mesh {
        Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION,self.position_vertices.clone(),)
        .with_inserted_indices(Indices::U32(self.indices.clone()))

    }

}