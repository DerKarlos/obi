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

// O SM  ///////////////////////

#[derive(Deserialize, Debug)]
struct JsonData {
    elements: Vec<JosnElement>,
}


struct OsmNode {
    lat: f64,
    lon: f64,
}


fn create_way (way_id: u64) -> Mesh {

    // https://www.openstreetmap.org/way/121486088#map=19/49.75594/11.13575&layers=D
    // window.open(httpx+"www.openstreetmap.org/way/121486088"_blank")
    // -           https://api.openstreetmap.org/api/0.6/way/121486088/full.json
    // https://master.apis.dev.openstreetmap.org/api/0.6/way/121486088/full.json does not have that way, 12148 works.
    // The test-server does not have needed objects (like Reifenberg), but they could be PUT into

    let url = format!("https://api.openstreetmap.org/api/0.6/way/{}/full.json", way_id);

    let json: JsonData = reqwest::blocking::get(url).unwrap().json().unwrap();

    //let nodes = Map(id:u64, node:OsmNode);
    let mut nodes_map = HashMap::new();

    type Position       = [f32;3];
    let mut position_vertices: Vec<Position> = vec![]; // 3 coordinates * x Positions. The corners are NOT reused to get hard Kanten
    let mut indices: Vec<u32> = vec![];


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
            println!("Way-center: lat = {:?} lon = {:?}", center_lat, center_lon );            

            for (index,node_id) in nodes.iter().enumerate() {
                let node = nodes_map.get(&node_id).unwrap();

                let x = ((node.lat - center_lat) * 10000.0) as f32;
                let z = ((node.lon - center_lon) * 10000.0) as f32;

                position_vertices.push( [x, 0.0, -z]);
                position_vertices.push( [x, 1.0, -z]);
                //println!("meter: index = {:?} latX = {:?} lonZ = {:?}", position_vertices.len(), x, z );

                if index>0 { // 1*2=2
                    indices.push((index*2+0) as u32);
                    indices.push((index*2+1) as u32);
                    indices.push((index*2-1) as u32);

                    indices.push((index*2-2) as u32);
                    indices.push((index*2+0) as u32);
                    indices.push((index*2-1) as u32);
                }

            }

        }
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        position_vertices,
    )
    .with_inserted_indices(Indices::U32(indices))

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

    let mesh_handle: Handle<Mesh> = meshes.add(create_way(reifenberg_id));
    commands.spawn((
        PbrBundle {
            mesh: mesh_handle,
            ..default()
        },
        Controled,
    ));

   // Transform for the camera and lighting, looking at (0,0,0) (the position of the mesh).
   let camera_and_light_transform =
        Transform::from_xyz(5., 5., 5.).looking_at(Vec3::ZERO, Vec3::Y);

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
        }
    }
    if keyboard_input.pressed(KeyCode::KeyY) {
        for mut transform in &mut query {
            transform.rotate_y(time.delta_seconds() / 1.2);
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

