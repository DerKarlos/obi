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

    let mut id = 664642004; //369161987; // 121486088 369161987; //159243621; // 437150850 //  121486088
    let show_only: u64 = 0; // 159243622; //1174629866; //1174306433;

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        dbg!(&args);
        id = args[1].parse().unwrap();
    }

    let api = InputLib::new(); // InputJson or InputLib
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
