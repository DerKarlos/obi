// other crates
use std::env;

#[cfg(feature = "json")]
use crate::input_osm_json::*;
#[cfg(feature = "xmllib")]
use crate::input_osm_lib::*;

use crate::bevy_ui::bevy_init;
use crate::kernel_in::{BoundingBox, LAT_FAKT};
use crate::to_3d::scan_objects;

///////////////////////////////////////////////////////////////////////////////////////////////////
// Example-Main: "OBI" directly by OSM-API Json ///////////////////////////////////////////////////
// Example-Main: "OBI" by LIB openstreetmap-api (XML) /////////////////////////////////////////////

pub async fn main_async() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "json")]
    let input = "Json";
    #[cfg(feature = "xmllib")]
    let input = "LIB";

    println!(
        "\n*********  Hi, I'm  O B I, or OSM-BI, the OSM Buiding Inspector ({input}) *********\n"
    );

    // Westminster 367642719, Abbey: 364313092
    // Passau Dom: 24771505
    // Reifenberg: 121486088
    // Bau 46:                                 Relation: 2819147 with Outer: 45590896 and  Inner: 210046607
    // St Paul's Cathedral: way 369161987 with Relation: 9235275 with Outer: 664646816  Dome: 664613340

    let mut id = 369161987;
    let show_only: u64 = 0; //629776388;

    let api = InputOsm::new();

    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        dbg!(&args);
        id = args[1].parse().unwrap();
    }

    let bounding_box = if args.len() == 5 {
        dbg!(&args);
        BoundingBox {
            west: args[1].parse().unwrap(),
            east: args[2].parse().unwrap(),
            north: args[3].parse().unwrap(),
            south: args[4].parse().unwrap(),
        }
    } else {
        api.geo_bbox_of_way(id).await?
    };
    println!("bounding_box: {:?}", bounding_box);

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
