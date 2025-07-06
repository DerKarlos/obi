// other crates
use clap::Parser;
use we_clap::WeParser; // Wrapper for clap Parser

// https://crates.io/crates/we_clap
#[derive(Parser, Debug, Default, Clone, Copy)]
#[command(about = "a minimal example of bevy_args", version, long_about = None)]
pub struct UrlClArgs {
    // Westminster 367642719, Abbey: 364313092
    // along: https://www.openstreetmap.org/way/363815745
    //
    // along!  RUST_BACKTRACE=1 cargo run --example m_async -- -w 363815745 -o 363815745
    // St Paul's Cathedral: way 369161987 with Relation: 9235'275 with Outer: 664646816  Dome: 664613340
    // Bau 46:                                 Relation: 2819'147 with Outer: 45590896 and  Inner: 210046607
    // Passau Dom: 24771505 = Outer
    // Reifenberg: 121486088
    // https://www.openstreetmap.org/query?lat=49.755930&lon=11.135741
    // https://www.openstreetmap.org/way/1174306435/history/3#map=19/49.755938/11.136073
    // https://demo.f4map.com/#lat=49.7556513&lon=11.1359205&zoom=21&camera.theta=55.936&camera.phi=-177.044
    // https://www.osmgo.org/v03.html?lat=49.7549930806153&lon=11.135770082473757&dir=0&view=-10&ele=101&multi=1
    // https://www.openstreetmap.org/edit?editor=id&way=1174306435#map=22/49.7558991/11.1357828
    //
    #[arg(short, long, default_value = "47942638")]
    pub way: u64,
    #[arg(short, long, default_value = "0")]
    pub only: u64,
    #[arg(short, long, default_value = "0")]
    pub range: f32,
}

// Implement web enabled parser for your struct
impl we_clap::WeParser for UrlClArgs {}

#[cfg(feature = "bevy")]
use crate::bevy_ui::render_init;
#[cfg(feature = "rend3")]
use crate::rend3_ui::render_init;

#[cfg(feature = "json")]
use crate::input_osm_json::*;
#[cfg(feature = "xmllib")]
use crate::input_osm_lib::*;

use crate::kernel_in::LAT_FAKT;
use crate::symbolic_3d::scan_elements_from_layer_to_mesh;

///////////////////////////////////////////////////////////////////////////////////////////////////
// Example-Main: "OBI" directly by OSM-API Json ///////////////////////////////////////////////////
// Example-Main: "OBI" by LIB openstreetmap-api (XML) /////////////////////////////////////////////

pub async fn main_async() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "json")]
    let input = "Json";
    #[cfg(feature = "xmllib")]
    let input = "LIB";

    println!(
        "\n*********  Hi, I'm  O B I, or OSM-BI, the OSM Buiding Inspector ({}) *********\n",
        input
    );

    // use web enabled parse and it works on native or web.
    let args: UrlClArgs = UrlClArgs::we_parse(); // Type annotations needed
    println!("{:?}", args);

    let api = InputOsm::new();

    let mut bounding_box = api.geo_bbox_of_way(args.way).await?;
    if args.range > 0. {
        bounding_box.min_range(args.range);
    }

    println!("bounding_box: {:?}", &bounding_box);

    let gpu_ground_null_coordinates = bounding_box.center_as_geographic_coordinates();
    let buildings_and_parts = api
        .scan_osm(&bounding_box, &gpu_ground_null_coordinates, args.only)
        .await?;
    // println!("buildings_and_parts: {:?}", buildings_and_parts);

    let meshes = scan_elements_from_layer_to_mesh(buildings_and_parts);
    let scale = bounding_box.max_radius() / 10. * LAT_FAKT;
    render_init(meshes, scale);

    Ok(())
}
