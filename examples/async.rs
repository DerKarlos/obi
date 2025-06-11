//// my crate
use osm_tb::*;

//// other crates
use std::env;

///////////////////////////////////////////////////////////////////////////////////////////////////
// Example-Main: "OBI" directly by OSM-API Json ///////////////////////////////////////////////////
// Example-Main: "OBI" by LIB openstreetmap-api (XML) /////////////////////////////////////////////

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "\n*********  Hi, I'm  O B I, or OSM-BI, the OSM Buiding Inspector (Json/LIB) *********\n"
    );

    // Reifenberg: 121486088
    // St Paul's Cathedral: 369161987 mit Relation: 9235275 mit Outer: 664646816

    let mut id = 369161987;
    let show_only: u64 = 0;

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        dbg!(&args);
        id = args[1].parse().unwrap();
    }

    let api = InputJson::new(); // InputJson or InputLib
    let bounding_box = api.geo_bbox_of_way(id).await?;

    let gpu_ground_null_coordinates = bounding_box.center_as_geographic_coordinates();
    let building_parts = api
        .scan_osm(&bounding_box, &gpu_ground_null_coordinates, show_only)
        .await?;
    // println!("building_parts: {:?}", building_parts);

    let meshes = scan_objects(building_parts);
    let scale = bounding_box.max_radius() / 10. * LAT_FAKT;
    bevy_init(meshes, scale);

    Ok(())
}
