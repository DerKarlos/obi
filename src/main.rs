mod bevy_ui;
mod input_json;
mod obi_api_in;
mod obi_api_out;
mod render_3d;

use bevy_ui::bevy_init;
use std::error::Error;

use crate::input_json::{coordinates_of_way, scan_json};
use crate::obi_api_in::GeographicCoordinates;
use crate::render_3d::scan_osm;

// use error_chain::error_chain;
// error_chain! {
//     foreign_links {
//         Io(std::io::Error);
//         HttpRequest(reqwest::Error);
//     }
// }

///////////////////////////////////////////////////////////////////////////////////////////////////
// MAIN ///////////////////////////////////////////////////////////////////////////////////////////

// https://github.com/DerKarlos/obi/tree/master/src

fn main() -> Result<(), Box<dyn Error>> {
    //fn main() -> Result<()> {
    println!("*********  Hi, I'm OBI, the OSM Buiding Inspector  *********");

    // Testing with this moderate complex building
    // https://www.openstreetmap.org/way/121486088#map=19/49.75594/11.13575&layers=D
    let _reifenberg_id = 121486088;
    let _westminster_id = 367642719;
    let id = _westminster_id;
    let scale = 10.0;
    let range = 15.0 * scale;

    let ground_null_coordinates = if true {
        // Todo: remove test
        coordinates_of_way(id)
    } else {
        // Default for Reifenberg:
        GeographicCoordinates {
            latitude: 49.755907953,
            longitude: 11.135770967,
        }
    };
    //println!("ground_null_coordinates: {:?}", &ground_null_coordinates);

    let buildings_or_parts = scan_json(&ground_null_coordinates, range);
    let osm_meshes = scan_osm(buildings_or_parts);

    bevy_init(osm_meshes, scale);

    Ok(())
}

// Todo: cargo clippy / run per key B for Build
