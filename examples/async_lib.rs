// Is the crate https://crates.io/crates/osm-api able to build wasm and deliver the needed data?
// If yes, a new input "modul" is needed. A good way to fine the division of the "OSM-Toolbox"

use std::env;

use osm_tb::*;

//#[cfg(target_arch = "wasm32")]
//pub async fn run() -> Result<(), Box<dyn std::error::Error>> {

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut id = 121486088 as u64;
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        dbg!(&args[1]);
        id = args[1].parse().unwrap();
    }

    let api = InputLib::new();
    let bounding_box = api.geo_bbox_of_way(id).await?;
    println!("{:?}", bounding_box);

    let gpu_ground_null_coordinates = bounding_box.center_as_geographic_coordinates();
    let building_parts = api
        .scan_osm(&bounding_box, &gpu_ground_null_coordinates, 0)
        .await?;
    println!("building_parts: {:?}", building_parts);

    let meshes = scan_objects(building_parts);
    let scale = bounding_box.max_radius() / 4. * LAT_FAKT;
    bevy_init(meshes, scale);

    Ok(())
}
