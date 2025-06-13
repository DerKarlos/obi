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

    // Westminster 367642719, Abbey: 364313092
    // Passau Dom: 24771505
    // Reifenberg: 121486088
    // Bau 46:                                 Relation: 2819147 with Outer: 45590896 and  Inner: 210046607
    // St Paul's Cathedral: way 369161987 with Relation: 9235275 with Outer: 664646816
    //   Dome: 664613340

    let mut id = 367642719;
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
