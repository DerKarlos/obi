// Varionus input modules are possible (OSM-Json, Vector-Tile-File, Overtures)

mod input_json;
//mod oma_reader;

// Interface from an input modules to a renderer
mod kernel_in;
mod shape;
// 3D and 2D rendere are possible
mod render_3d;
mod tagticks;
// Interface from an rederer to an output
mod kernel_out;
// Variouns outputs are possible (UI, create a GLB file)
mod bevy_ui;
//mod f4control;

// other crates
use bevy_ui::bevy_init;
use input_json::coordinates_of_way_center;
use std::env;
use std::error::Error;
// this crate
use crate::input_json::{get_range_json, scan_json};
use crate::kernel_in::LAT_FAKT;
use crate::render_3d::scan_objects;
// Todo? use error_chain::error_chain;

/**** Project patterns ****************************************************************************
 * Don't use apreviations, as Rust does
 * Always north before east, like in GroundPosition
 * Now and then check for all clone() and copy() to be realy needed
 */

/**** TODO ***************************************************************************************
 * Wesminster has some odd parts underground: OBI error: https://www.openstreetmap.org/way/1141764452  Kommented on Changeset: https://www.openstreetmap.org/changeset/132598262?xhr=1
 */

///////////////////////////////////////////////////////////////////////////////////////////////////
// MAIN / Example: "OBI" //////////////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("\n*********  Hi, I'm  O B I, the OSM Buiding Inspector  *********\n");

    std::env::set_var("RUST_LIB_BACKTRACE", "1");

    let args: Vec<String> = env::args().collect();

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

    let mut id = _westminster_id as u64;
    let show_only: u64 = 0;

    if args.len() > 1 {
        dbg!(&args[1]);
        id = args[1].parse().unwrap();
    }

    let (ground_null_coordinates, bounding_box) = coordinates_of_way_center(id).await;

    println!("Center is id {} at: {:?}\n", id, &ground_null_coordinates);

    let scale = bounding_box.max_radius() / 4. * LAT_FAKT;

    let range_json = get_range_json(bounding_box);
    //println!("range_json: {:?}", range_json);
    let building_parts = scan_json(range_json, &ground_null_coordinates, show_only);
    //println!("building_parts: {:?}", building_parts);
    let meshes = scan_objects(building_parts);
    bevy_init(meshes, scale);

    Ok(())
}
