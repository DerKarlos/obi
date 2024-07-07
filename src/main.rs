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
    building: Option<String>,
    name: Option<String>,
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


fn _geopos_of_way (way_id: u64) -> GeoPos {

    // DONT USE:   https://api.openstreetmap.org/api/0.6/way/121486088/full.json
    // https://master.apis.dev.openstreetmap.org/api/0.6/way/121486088/full.json
    // The test-server does not have needed objects (like Reifenberg), but they could be PUT into

    let url = format!("https://api.openstreetmap.org/api/0.6/way/{}/full.json", way_id);

    let json: JsonData = reqwest::blocking::get(url).unwrap().json().unwrap();

    let mut lat: f64 = 0.0;
    let mut lon: f64 = 0.0;
    let mut nodes: f64 = 0.;

    // add the coordinates of oll nodes
    for element in json.elements {
        if element.element_type == "node".to_string() {
            nodes += 1.0;
            lat += element.lat.unwrap();
            lon += element.lon.unwrap();
        }

    }
    // calculate and return everedge
    lat /= nodes;
    lon /= nodes;
    println!("geopos_of_way: lat = {:?} lon = {:?}", lat, lon );
    GeoPos{lat,lon}

}

fn create_way  (geo_pos_at_gpu0: GeoPos, way_id: u64) -> Mesh {

    // TODO:  AREA !!!!
    let url = format!("https://api.openstreetmap.org/api/0.6/way/{}/full.json", way_id);
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
            let id = element.id;
            let nodes = element.nodes.unwrap();
            let tags = element.tags.unwrap();
            let name = tags.name.unwrap();
            let building = tags.building.unwrap();

            println!(" Way: id = {:?}  building = {:?}  name = {:?}",id,building,name,);

            // Get building walls from nodes
            let mut last_pos_down = [0.0,0.0,0.0];
            let mut last_pos_up   = [0.0,0.0,0.0];

            for (index,node_id) in nodes.iter().enumerate() {
                let node = nodes_map.get(&node_id).unwrap();

                let this_pos_down = [node.pos.x,0.0,node.pos.z];
                let this_pos_up   = [node.pos.x,10.,node.pos.z];

                if index>0 {
                    cmesh.push_square(
                        last_pos_down,
                        this_pos_down,
                        last_pos_up,
                        this_pos_up,
                    );
                }
                last_pos_down = this_pos_down;
                last_pos_up   = this_pos_up;
            }

        }
    }

    cmesh.get_mesh()

}


// MAIN ///////////////////

fn main() -> Result<()>
{     
    println!("Hi, I'm OBI, the OSM Buiding Inspector");

    // BEVY-App
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, input_handler)
        .run();

    Ok(())

}

// BEVY ////////////////////

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {

    // Testing with this moderate complex building
    // https://www.openstreetmap.org/way/121486088#map=19/49.75594/11.13575&layers=D
    let reifenberg_id = 121486088;

    // get geo-position at the 0 position of the GPU
    let geo_pos_at_gpu0 = // geopos_of_way(reifenberg_id);
                          GeoPos{lat: 49.75590795333333, lon: 11.135770966666666};
    commands.insert_resource(GeoPosAtGPU0{_pos: geo_pos_at_gpu0.clone()});


    // Dodo: get building ... other way
    let mesh_handle: Handle<Mesh> = meshes.add(create_way(geo_pos_at_gpu0, reifenberg_id));
    commands.spawn((
        PbrBundle {
            mesh: mesh_handle,
            ..default()
        },
        Controled,
    ));

   // Transform for the camera and lighting, looking at (0,0,0) (the position of the mesh).
   let camera_and_light_transform =
        Transform::from_xyz(50., 50., 50.).looking_at(Vec3::ZERO, Vec3::Y);

    // Camera in 3D space.
    commands.spawn(Camera3dBundle {
        transform: camera_and_light_transform,
        ..default()
    });

    // Light up the scene.
    commands.spawn(PointLightBundle {
        transform: camera_and_light_transform,
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
            println!("Key X");
        }
    }
    if keyboard_input.pressed(KeyCode::KeyY) {
        for mut transform in &mut query {
            transform.rotate_y(time.delta_seconds() / 1.2);
            println!("Key y");
        }
    }
    if keyboard_input.pressed(KeyCode::KeyZ) {
        for mut transform in &mut query {
            transform.rotate_z(time.delta_seconds() / 1.2);
            println!("Key Z");
        }
    }
    if keyboard_input.pressed(KeyCode::KeyR) {
        for mut transform in &mut query {
            transform.look_to(Vec3::NEG_Z, Vec3::Y);
            println!("Key R");
        }
    }
}


// ??? //////////////////////

type Position = [f32;3];

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
    position_vertices: Vec<Position>, // 3 coordinates * x Positions. The corners are NOT reused to get hard Kanten
    indices: Vec<u32>,
}

impl CMesh {

    pub fn new() -> Self {
        Self{
            position_vertices: vec![],
            indices: vec![],
        }
    }

    pub fn _push(&mut self, index: usize, x: f32, z: f32) {
        self.position_vertices.push( [x, 0.0, -z]);
        self.position_vertices.push( [x, 1.0, -z]);
        //println!("meter: index = {:?} latX = {:?} lonZ = {:?}", position_vertices.len(), x, z );

        if index>0 { // 1*2=2
            self.indices.push((index*2+0) as u32);
            self.indices.push((index*2+1) as u32);
            self.indices.push((index*2-1) as u32);

            self.indices.push((index*2-2) as u32);
            self.indices.push((index*2+0) as u32);
            self.indices.push((index*2-1) as u32);
        }

    }

    pub fn push_square(&mut self, down_left: Position, down_right: Position, up_left: Position, up_right: Position  ) {
        // First index of the comming 4 positions
        let index = self.position_vertices.len();
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
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION,self.position_vertices.clone(),)
        .with_inserted_indices(Indices::U32(self.indices.clone()))

    }

}