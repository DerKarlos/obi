// Varionus input modules are possible (OSM-Json, Vector-Tile-File, Overtures)
mod input_json;
// Interface from an input modules to a renderer
mod api_in;
// 3D and 2D rendere are possible
mod render_3d;
// Interface from an rederer to an output
mod api_out;
// Variouns outputs are possible (UI, create a GLB file)
mod bevy_ui;

// other crates
use bevy_ui::bevy_init;
use input_json::coordinates_of_way_center;
use std::error::Error;
// this crate
use crate::input_json::{get_json_range, scan_json};
use crate::render_3d::scan_osm;
// Todo? use error_chain::error_chain;

/**** Project patterns ****************************************************************************
 * Don't use apreviations, as Rust does
 * Always north before east, like in GroundPosition
 */

///////////////////////////////////////////////////////////////////////////////////////////////////
// MAIN / Example: "OBI" //////////////////////////////////////////////////////////////////////////

fn main() -> Result<(), Box<dyn Error>> {
    println!("\n*********  Hi, I'm  O B I, the OSM Buiding Inspector  *********\n");

    // Testing with a moderate complex building OR a lage complex one
    // https://www.openstreetmap.org/way/121486088#map=19/49.75594/11.13575&layers=D
    let _reifenberg_id = 121486088;
    let _westminster_id = 367642719;
    let _passau_dom_id = 24771505;
    let id = _passau_dom_id;
    let scale = 5.0;
    let range = 15.0 * scale;

    let ground_null_coordinates = coordinates_of_way_center(id);
    println!("ground_null_coordinates: {:?}", &ground_null_coordinates);

    let json_data = get_json_range(range, &ground_null_coordinates);
    let building_parts = scan_json(json_data, &ground_null_coordinates);
    let osm_meshes = scan_osm(building_parts);

    bevy_init(osm_meshes, scale);

    Ok(())
}
