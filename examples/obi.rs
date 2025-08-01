// other crates:
use clap::Parser;
use we_clap::WeParser; // Wrapper for clap Parser

// own lib:
use osm_tb::*;

// https://crates.io/crates/we_clap
#[derive(Parser, Debug, Default, Clone, Copy)]
#[command(about = "a minimal example of bevy_args", version, long_about = None)]
pub struct UrlClArgs {
    // Westminster 367642719, Abbey: 364313092
    // Passau Dom: 24771505 = Outer | Reifenberg: 121486088 | Krahnhaus:234160726
    // St Paul's Cathedral: way 369161987 with Relation: 9235'275 with Outer: 664646816  Dome: 664613340
    #[arg(short, long, default_value = "369161987")] //
    pub way: u64,
    #[arg(short, long, default_value = "0")]
    pub only: u64,
    #[arg(short, long, default_value = "0")]
    pub range: f32,
}

// Implement web enabled parser for your struct
impl we_clap::WeParser for UrlClArgs {}

////////////////////////////////////////////////////////////////////////
// Example: "OBI" async directly by OSM-API Json                      //
//                            or by LIB openstreetmap-api (XML)       //
////////////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    let range_string = if args.way > 0 {
        format!("(range {})", args.range)
    } else {
        "".into()
    };
    println!("Inspecting way {} {range_string}", args.way); // Not Info! from Bevy because this sourc sould work without Bevy to. Like with rend3

    let api = InputOsm::new();

    let bounding_box = api.geo_bbox_of_way(args.way).await;
    let mut bounding_box = match bounding_box {
        Ok(bounding_box) => bounding_box,
        Err(e) => {
            panic!("Way loading Error: {e:?}");
        }
    };

    if bounding_box.north == 0.0 {
        return Ok(());
    }

    bounding_box.min_range(args.range);
    let range = bounding_box.max_radius() * LAT_FAKT as f32;
    #[cfg(debug_assertions)]
    println!("bounding_box: {:?}", &bounding_box);
    println!("Loading data");

    let gpu_ground_null_coordinates = bounding_box.center_as_geographic_coordinates();
    let buildings_and_parts = api
        .scan_osm(&bounding_box, &gpu_ground_null_coordinates, args.only)
        .await?;
    // println!("buildings_and_parts: {:?}", buildings_and_parts);

    if buildings_and_parts.is_empty() {
        println!("No building(s)");
        return Ok(());
    }

    println!("Rendering ...\n");
    let meshes = scan_elements_from_layer_to_mesh(buildings_and_parts);
    render_init(
        meshes, range, true, /* use first mouse key for orientation */
    );

    Ok(())
}
