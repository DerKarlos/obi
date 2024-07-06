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
    tags: Option<Tags>,
}

#[derive(Deserialize, Debug)]
struct Tags {
    building: Option<String>,
    name: Option<String>,
}

#[derive(Deserialize, Debug)]
struct JsonData {
    elements: Vec<JosnElement>,
}

// OSM ///////////////////////

#[derive(Resource)]
struct GPSatGPU0 {
    pub lat: f64,
    pub lon: f64,
}

struct OsmNode {
    lat: f64,
    lon: f64,
}


fn create_way ( commands: &mut Commands, way_id: u64) -> Mesh {

    // https://www.openstreetmap.org/way/121486088#map=19/49.75594/11.13575&layers=D
    // window.open(httpx+"www.openstreetmap.org/way/121486088"_blank")
    // -           https://api.openstreetmap.org/api/0.6/way/121486088/full.json
    // https://master.apis.dev.openstreetmap.org/api/0.6/way/121486088/full.json does not have that way, 12148 works.
    // The test-server does not have needed objects (like Reifenberg), but they could be PUT into

    let url = format!("https://api.openstreetmap.org/api/0.6/way/{}/full.json", way_id);

    let json: JsonData = reqwest::blocking::get(url).unwrap().json().unwrap();

    //let nodes = Map(id:u64, node:OsmNode);
    let mut nodes_map = HashMap::new();

    let mut cmesh = CMesh::new();

    for element in json.elements {
        if element.element_type == "node".to_string() {
            let osm_node = OsmNode{lat: element.lat.unwrap(), lon: element.lon.unwrap(),};
            nodes_map.insert(element.id, osm_node);
            // println!("Node: id = {:?} lat = {:?} lon = {:?}", element.id, element.lat.unwrap(), element.lon.unwrap() );
        }
        if element.element_type == "way".to_string() {
            let id = element.id;
            let nodes = element.nodes.unwrap();
            let tags = element.tags.unwrap();
            let name = tags.name.unwrap();
            let building = tags.building.unwrap();

            println!(" Way: id = {:?}  building = {:?}  name = {:?}",
                id,
                building,
                name,
            );

            let mut lat_min: f64 = 1e9;
            let mut lon_min: f64 = 1e9;
            let mut lat_max: f64 = -1e9;
            let mut lon_max: f64 = -1e9;

            for node_id in &nodes {
                let node = nodes_map.get(&node_id).unwrap();

                lat_min = lat_min.min(node.lat);
                lat_max = lat_max.max(node.lat);
                lon_min = lon_min.min(node.lon);
                lon_max = lon_max.max(node.lon);
                //println!("Way-Node: id = {:?} lat = {:?} lon = {:?}", node_id, node.lat, node.lon );
            }
            let center_lat = lat_min + (lat_max-lat_min)/2.0;
            let center_lon = lon_min + (lon_max-lon_min)/2.0;

            let gps_at_gpu0 = GPSatGPU0{lat: center_lat, lon: center_lon};

            println!("Way-center: lat = {:?} lon = {:?}", center_lat, center_lon );

            let mut last_pos_down = [0.0,0.0,0.0];
            let mut last_pos_up   = [0.0,0.0,0.0];

            for (index,node_id) in nodes.iter().enumerate() {
                let node = nodes_map.get(&node_id).unwrap();

                let this_pos_down = to_position( &gps_at_gpu0, node.lat, node.lon, 0.00);
                let mut this_pos_up   = this_pos_down;
                this_pos_up[1] = 10.0;

                // cmesh.push(index,x,z);

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

            commands.insert_resource(gps_at_gpu0);

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

    let reifenberg_id = 121486088;

    let mesh_handle: Handle<Mesh> = meshes.add(create_way(&mut commands, reifenberg_id));
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

fn to_position( gps_at_gpu0: &GPSatGPU0, lat: f64, lon: f64, height: f64) -> Position {
    // the closer to the pole, the smaller the tiles size in meters get
    let lon_fakt = LAT_FAKT * ((lat / 180. * PI as f64).abs()).cos(); // Longitude(Längengrad) West/East factor
    // actual geo pos - other geo pos = relative geo/meter scene pos
    let x = (lon - gps_at_gpu0.lon) * lon_fakt;
    let z = (lat - gps_at_gpu0.lat) * LAT_FAKT;
    /*return*/ [x as f32, height as f32, z as f32]
}

//    pub fn calc_scene_pos(lat: f64, lon: f64, osm_scene: &OsmScene) -> Position {
//        let pos_relative_to_corner = self.calc_meters_to_other_geo_pos(osm_scene.null_corner_geo_pos);
//        let mut scene_pos = pos_relative_to_corner
//        scene_pos.x =  (scene_pos.x * 100.).floor() / 100.; // cm is acurate in this case
//        scene_pos.z = -(scene_pos.z * 100.).floor() / 100.; // !!!!! The - is NOT clear, but works :-/
//        scene_pos // gps-degrees plus/nord = z-meter plus/behind "In the backround = north"
//    }


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