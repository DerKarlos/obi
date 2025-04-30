use csscolorparser::parse;
use serde::Deserialize;
use std::collections::HashMap;

use crate::api_in::{
    BuildingPart, GeographicCoordinates, GroundPosition, OsmNode, RenderColor, RoofShape,
};

///////////////////////////////////////////////////////////////////////////////////////////////////
// JOSN ///////////////////////////////////////////////////////////////////////////////////////////

static YES: &str = "yes";
static NO: &str = "no";

static API_URL: &str = "https://api.openstreetmap.org/api/0.6/";

/* Factor to calculate meters from gps coordiantes.decimals (latitude, Nort/South position) */
static LAT_FAKT: f64 = 111100.0; // 111285; // exactly enough  111120 = 1.852 * 1000.0 * 60  // 1 NM je Bogenminute: 1 Grad Lat = 60 NM = 111 km, 0.001 Grad = 111 m
static PI: f32 = std::f32::consts::PI;

static DEFAULT_WALL_COLOR: &str = "grey"; // RenderColor = [0.5, 0.5, 0.5, 1.0]; // "grey"
static DEFAULT_ROOF_COLOR: &str = "red"; // RenderColor = [1.0, 0.0, 0.0, 1.0]; // "red"
static DEFAULT_WALL_HEIGHT: f32 = 2.0 * 3.0; // two floors with each 3 meters
static DEFAULT_ROOF_HEIGHT: f32 = 0.0;

// todo: &str   https://users.rust-lang.org/t/requires-that-de-must-outlive-static-issue/91344/10
#[derive(Deserialize, Debug)]
struct JosnElement {
    #[serde(rename = "type")]
    element_type: String,
    id: u64,
    lat: Option<f64>,
    lon: Option<f64>,
    nodes: Option<Vec<u64>>,
    tags: Option<JosnTags>, // todo?: use a map
}

#[derive(Deserialize, Debug, Clone, Default)]
struct JosnTags {
    // name: Option<String>,
    // building: Option<String>,
    #[serde(rename = "building:part")]
    building_part: Option<String>, // ??? &'static str>,
    #[serde(rename = "roof:shape")]
    roof_shape: Option<String>,
    #[serde(rename = "roof:colour")]
    roof_color: Option<String>,
    #[serde(rename = "colour")]
    color: Option<String>,
    #[serde(rename = "roof:height")]
    roof_height: Option<String>,
    height: Option<String>,
    min_height: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct JsonData {
    elements: Vec<JosnElement>,
}

fn parse_height(height: Option<String>, default: f32) -> f32 {
    if height.is_none() {
        return default;
    }

    match height.unwrap().as_str().parse() {
        Ok(height) => height,

        Err(error) => {
            println!("Error! parse_height: {}", error);
            DEFAULT_ROOF_HEIGHT
        }
    }
}

fn parse_color(color: String) -> RenderColor {
    // Bevy pbr color needs f32, The parse has no .to_f32_array???}
    // https://docs.rs/csscolorparser/latest/csscolorparser/
    match parse(color.as_str()) {
        Ok(color_scc) => [
            color_scc.r as f32,
            color_scc.g as f32,
            color_scc.b as f32,
            color_scc.a as f32,
        ],

        Err(error) => {
            println!("parse_colour: {}", error);
            [0.5, 0.5, 1.0, 1.0] // "light blue?"
        }
    }
}

fn to_position(
    ground_null_coordinates: &GeographicCoordinates,
    latitude: f64,
    longitude: f64,
) -> GroundPosition {
    if ground_null_coordinates.latitude == 0. {
        return GroundPosition {
            north: latitude as f32,
            east: longitude as f32,
        };
    }

    // the closer to the pole, the smaller the tiles size in meters get
    let lon_fakt = LAT_FAKT * ((latitude / 180. * PI as f64).abs()).cos();
    // Longitude(Längengrad) West/East factor
    // actual coor - other coor = relative grad/meter ground position

    let north = ((latitude - ground_null_coordinates.latitude) * LAT_FAKT) as f32;
    let east = ((longitude - ground_null_coordinates.longitude) * lon_fakt) as f32;

    GroundPosition { north, east }
}

/**/
pub fn coordinates_of_way_center(way_id: i64) -> GeographicCoordinates {
    // DONT USE:   https://api.openstreetmap.org/api/0.6/way/121486088/full.json
    // https://master.apis.dev.openstreetmap.org/api/0.6/way/121486088/full.json
    // The test-server does not have needed objects (like Reifenberg), but they could be PUT into

    let url = format!("{}way/{}/full.json", API_URL, way_id);

    // Get OSM data from API and convert Json to Rust types. See https://serde.rs
    let json_way: JsonData = reqwest::blocking::get(url).unwrap().json().unwrap();

    let mut latitude: f64 = 0.0;
    let mut longitude: f64 = 0.0;
    let mut nodes_divider: f64 = -1.;

    // add the coordinates of all nodes
    for element in json_way.elements {
        if element.element_type == "node" {
            if nodes_divider >= 0. {
                latitude += element.lat.unwrap();
                longitude += element.lon.unwrap();
            }
            nodes_divider += 1.0;
        }
    }
    // calculate and return everedge
    latitude /= nodes_divider;
    longitude /= nodes_divider;
    GeographicCoordinates {
        latitude,
        longitude,
    }
}
/**/

fn node(
    element: JosnElement,
    ground_null_coordinates: &GeographicCoordinates,
    nodes_map: &mut HashMap<u64, OsmNode>,
) {
    let osm_node = OsmNode {
        position: to_position(
            ground_null_coordinates,
            element.lat.unwrap(),
            element.lon.unwrap(),
        ),
    };
    nodes_map.insert(element.id, osm_node);
    // println!("Node: id = {:?} lat = {:?} lon = {:?}", element.id, element.lat.unwrap(), element.lon.unwrap() );
}

fn way(
    element: JosnElement,
    building_parts: &mut Vec<BuildingPart>,
    nodes_map: &mut HashMap<u64, OsmNode>,
) {
    // println!("element = {:?}", element);
    //let tags_option = element.tags.unwrap(); // JosnTags { ..default() }; //ttt

    if element.tags.is_none() {
        println!("way without tags! ID: {} Multipolligon?", element.id);
        return;
    }

    let tags = element.tags.as_ref().unwrap(); // JosnTags { ..default() }; //ttt
    let string_no = &NO.to_string();
    let part = tags.building_part.as_ref().unwrap_or(string_no);

    // ??? not only parts!
    if part == YES {
        building(element, building_parts, nodes_map);
    };
}

fn building(
    element: JosnElement,
    building_parts: &mut Vec<BuildingPart>,
    nodes_map: &mut HashMap<u64, OsmNode>,
) {
    // println!(" Way: building = {:?}  name = {:?}" name,);
    let tags = element.tags.unwrap(); // JosnTags { ..default() }; //ttt

    // Colors and Materials
    let color = parse_color(tags.color.unwrap_or(DEFAULT_WALL_COLOR.to_string()));
    let roof_color = parse_color(tags.roof_color.unwrap_or(DEFAULT_ROOF_COLOR.to_string()));

    // Heights
    let min_height = parse_height(tags.min_height, 0.0);
    let roof_height = parse_height(tags.roof_height, DEFAULT_ROOF_HEIGHT);
    let wall_height = parse_height(tags.height, DEFAULT_WALL_HEIGHT) - roof_height;

    // Shape of the roof. All buildings have a roof, even if it is not tagged
    let roof_shape = tags.roof_shape;
    let shape: RoofShape = match roof_shape {
        None => RoofShape::None,
        Some(shape) => match shape.as_str() {
            "flat" => RoofShape::Flat,
            "onion" => RoofShape::Onion,
            "pyramidal" => RoofShape::Phyramidal,
            _ => {
                println!("Warning: roof_shape Unknown: {}", shape);
                RoofShape::Flat // todo: gabled and geographic dependend
            }
        },
    };

    // Get building footprint from nodes
    let nodes = element.nodes.unwrap();
    if nodes.len() < 3 {
        println!("Building with < 3 corners! id: {}", element.id);
        return;
    }
    if nodes.first().unwrap() != nodes.last().unwrap() {
        println!("Building with < 3 corners! id: {}", element.id);
    }
    // else { todo("drop last and modulo index") }

    let mut sum_north = 0.;
    let mut sum_east = 0.;
    let mut _longest_side_length = 0;
    let mut _longest_side_index = 0;
    let mut footprint: Vec<GroundPosition> = Vec::new();
    for (index, node_id) in nodes.iter().rev().enumerate() {
        //r (index, position) in building_part.footprint.iter().rev().enumerate() {
        let node = nodes_map.get(node_id).unwrap();
        footprint.push(node.position);
        if index > 0 {
            sum_east += node.position.east;
            sum_north += node.position.north;
        }
    }
    let count = nodes.len() as f32 - 1.;
    let center = GroundPosition {
        north: sum_north / count,
        east: sum_east / count,
    };
    println!("roof_shape: {:?}", shape);
    let building_part = BuildingPart {
        _part: true, // ??? not only parts!
        footprint,
        _longest_side_index,
        center,
        wall_height,
        min_height,
        color,
        roof_shape: shape,
        roof_height,
        roof_color,
    };

    building_parts.push(building_part);
}

pub fn _get_json_way(way_id: i64) -> JsonData {
    let url = format!("{}way/{}/full.json", API_URL, way_id);
    reqwest::blocking::get(url).unwrap().json().unwrap()
}

pub fn get_json_range(range: f64, ground_null_coordinates: &GeographicCoordinates) -> JsonData {
    // https://wiki.openstreetmap.org/wiki/API_v0.6#Retrieving_map_data_by_bounding_box:_GET_/api/0.6/map
    let range = range / LAT_FAKT; // First test with 15 meter
    let left = ground_null_coordinates.longitude - range;
    let right = ground_null_coordinates.longitude + range;
    let top = ground_null_coordinates.latitude + range;
    let bottom = ground_null_coordinates.latitude - range;
    // let left_top = to_position(&CoordinatesAtGroundPositionNull, left, top);
    // println!("range: left_top={} url={}", left_top, url);
    // GET   /api/0.6/map?bbox=left,bottom,right,top
    let url = format!(
        "{}map.json?bbox={},{},{},{}",
        API_URL, left, bottom, right, top
    );

    // let json_map: JsonData = reqwest::blocking::get(url).unwrap().json().unwrap();
    reqwest::blocking::get(url).unwrap().json().unwrap()
}

pub fn scan_json(
    json_data: JsonData,
    ground_null_coordinates: &GeographicCoordinates,
) -> Vec<BuildingPart> {
    let mut building_parts: Vec<BuildingPart> = Vec::new();
    let mut nodes_map = HashMap::new();

    for element in json_data.elements {
        // println!("element.element_type: {}", element.element_type);
        match element.element_type.as_str() {
            "node" => node(element, ground_null_coordinates, &mut nodes_map),
            "way" => way(element, &mut building_parts, &mut nodes_map),
            _ => println!(
                "Error: Unknown element type: {}  id: {}",
                element.element_type, element.id
            ),
        }
    }

    building_parts
}
