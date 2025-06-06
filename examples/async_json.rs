// This is the same code as it was in src/main.rs
// The affect of having a lib.rs AND a main.rs is that both a binary and library are compiled.

//// my crate
use osm_tb::*;

//// other crates
use reqwest;
//use serde::*;
//use serde_json::*;
use std::env;
use std::error::Error;

///////////////////////////////////////////////////////////////////////////////////////////////////
// MAIN / Example: "OBI" //////////////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("\n*********  Hi, I'm  O B I, or OSM-BI, the OSM Buiding Inspector  *********\n");
    // edition = "2024": error[E0133]: call to unsafe function `set_var` is unsafe and requires unsafe block
    // std::env::set_var("RUST_LIB_BACKTRACE", "1");

    // Testing with a moderate complex building OR a lage complex one
    // https://www.openstreetmap.org/way/121486088#map=19/49.75594/11.13575&layers=D
    let _reifenberg_id = 121486088; // scale 5
    let _passau_dom_id = 24771505; // scale 15   gabled: 464090146   unten: 136144290  oben: 136144289
    let _westminster_id = 367642719; // 25
    let _taj_mahal_id = 375257537;
    let _marienplatz_id = 223907278; // 15
    let _fo_gabled = 45696973; // rectangle: 47942624 +schräg: 45697283  haustür: 47942638  eingeeckt: 45697162  winklel: 45402141
    // no roof 45697280 BADs!: 45697037, 45402130  +OK+: 37616289
    // Not valide tagged???: 45696973

    let mut id = 121486088 as u64;
    let show_only: u64 = 0;

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        dbg!(&args[1]);
        id = args[1].parse().unwrap();
    }

    // Get the center of the GPU scene
    let url = way_url(id);
    //let response = reqwest::get(url).await?;
    let bytes = reqwest::get(url).await?.bytes().await?;
    //ok: let json_way_data: JsonData = response.json().await?;
    // let bytes = response.bytes().await?;
    // let json_way_data: JsonData = serde_json::from_slice(bytes).unwrap();
    // let bounding_box = geo_bbox_of_way_json(json_way_data);
    let bounding_box = geo_bbox_of_way_bytes(&bytes);
    let gpu_ground_null_coordinates = bounding_box.center_as_geographic_coordinates();
    println!(
        "Center is id {} at: {:?}\n",
        id, &gpu_ground_null_coordinates
    );

    //// Get OSM data and convert Json to Rust types. See https://serde.rs
    let url = bbox_url(&bounding_box);
    // println!("url: {url}");
    let bytes = reqwest::get(url).await?.bytes().await?;
    // println!("json_bbox_data: {:?}", json_bbox_data);
    let building_parts = scan_osm_bytes(bytes, &gpu_ground_null_coordinates, show_only);
    //println!("building_parts: {:?}", building_parts);
    let meshes = scan_objects(building_parts);

    let scale = bounding_box.max_radius() / 10. * LAT_FAKT;
    bevy_init(meshes, scale);
    /****/

    Ok(())
}
