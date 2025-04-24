use std::error::Error;
// use error_chain::error_chain;

mod bevy_ui;
mod json_osm;
use bevy_ui::bevy_init;

use crate::json_osm::{coordinates_of_way, scan_json, GeographicCoordinates};

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
    let reifenberg_id = 121486088;

    let ground_null_coordinates = if false {
        // Todo: remove test
        coordinates_of_way(reifenberg_id)
    } else {
        // Default for Reifenberg:
        GeographicCoordinates {
            latitude: 49.755907953,
            longitude: 11.135770967,
        }
    };
    //println!("ground_null_coordinates: {:?}", &ground_null_coordinates);

    let osm_meshes = scan_json(&ground_null_coordinates);

    bevy_init(osm_meshes);

    Ok(())
}
