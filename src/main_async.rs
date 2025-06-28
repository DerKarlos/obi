// other crates
use clap::Parser;
use we_clap::WeParser; // Wrapper for clap Parser

// https://crates.io/crates/we_clap
#[derive(Parser, Debug, Default, Clone, Copy)]
#[command(about = "a minimal example of bevy_args", version, long_about = None)]
pub struct UrlClArgs {
    // Westminster 367642719, Abbey: 364313092
    // along!  RUST_BACKTRACE=1 cargo run --example m_async -- -w 363815745 -o 363815745
    // St Paul's Cathedral: way 369161987 with Relation: 9235'275 with Outer: 664646816  Dome: 664613340
    // Bau 46:                                 Relation: 2819'147 with Outer: 45590896 and  Inner: 210046607
    // Passau Dom: 24771505
    // Reifenberg: 121486088
    //
    #[arg(short, long, default_value = "369161987")]
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
use crate::symbolic_3d::scan_objects;

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
    let building_parts = api
        .scan_osm(&bounding_box, &gpu_ground_null_coordinates, args.only)
        .await?;
    // println!("building_parts: {:?}", building_parts);

    let meshes = scan_objects(building_parts);
    let scale = bounding_box.max_radius() / 10. * LAT_FAKT;
    render_init(meshes, scale);

    Ok(())
}
