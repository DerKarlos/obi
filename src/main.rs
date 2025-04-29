mod api_in;
mod api_out;
mod bevy_ui;
mod input_json;
mod render_3d;

use bevy_ui::bevy_init;
use input_json::coordinates_of_way_center;
use std::error::Error;

//e crate::api_in::GeographicCoordinates;
use crate::input_json::{get_json_range, scan_json};
use crate::render_3d::scan_osm;
// Todo? use error_chain::error_chain;

///////////////////////////////////////////////////////////////////////////////////////////////////
// MAIN / Example: "OBI" //////////////////////////////////////////////////////////////////////////

fn main() -> Result<(), Box<dyn Error>> {
    println!("\n*********  Hi, I'm  O B I, the OSM Buiding Inspector  *********\n");

    // Testing with a moderate complex building OR a lage complex one
    // https://www.openstreetmap.org/way/121486088#map=19/49.75594/11.13575&layers=D
    let _reifenberg_id = 121486088;
    let _westminster_id = 367642719;
    let id = _reifenberg_id;
    let scale = 2.0;
    let range = 15.0 * scale;

    let ground_null_coordinates = coordinates_of_way_center(id);
    println!("ground_null_coordinates: {:?}", &ground_null_coordinates);

    let json_data = get_json_range(range, &ground_null_coordinates);
    let buildings_or_parts = scan_json(json_data, &ground_null_coordinates);
    let osm_meshes = scan_osm(buildings_or_parts);

    bevy_init(osm_meshes, scale);

    Ok(())
}
