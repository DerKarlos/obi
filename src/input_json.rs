use csscolorparser::parse;
use serde::Deserialize;
use std::collections::HashMap;

use crate::api_in::{
    BuildingOrPart, GeographicCoordinates, GroundPosition, OsmNode, RenderColor, Roof, RoofShape,
};

///////////////////////////////////////////////////////////////////////////////////////////////////
// JOSN ///////////////////////////////////////////////////////////////////////////////////////////

static YES: &str = "yes";
static NO: &str = "no";

static API_URL: &str = "https://api.openstreetmap.org/api/0.6/";

static PI: f32 = std::f32::consts::PI;
static LAT_FAKT: f64 = 111100.0; // 111285; // exactly enough  111120 = 1.852 * 1000.0 * 60  // 1 NM je Bogenminute: 1 Grad Lat = 60 NM = 111 km, 0.001 Grad = 111 m
/** Factor to calculate meters from gps coordiantes.decimals (latitude, Nort/South position) */

// todo: &str   https://users.rust-lang.org/t/requires-that-de-must-outlive-static-issue/91344/10
#[derive(Deserialize, Debug)]
struct JosnElement {
    #[serde(rename = "type")]
    element_type: String,
    id: u64,
    lat: Option<f64>,
    lon: Option<f64>,
    nodes: Option<Vec<u64>>,
    tags: Option<JosnTags>, // todo: use a map
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

fn parse_height(height: Option<String>) -> Option<f32> {
    //if
    height.as_ref()?;
    //    is_none() {
    //    return None;
    //};

    match height.unwrap().as_str().parse() {
        Ok(height) => Some(height),

        Err(error) => {
            println!("parse_height: {}", error);
            None
        }
    }
}

fn parse_color(color: Option<String>) -> Option<RenderColor> {
    //if color.is_none() {
    //    return None;
    //};
    color.as_ref()?;

    // Bevy pbr color needs f32, The parse has no .to_f32_array???}
    // https://docs.rs/csscolorparser/latest/csscolorparser/
    match parse(color.unwrap().as_str()) {
        Ok(color_scc) => Some([
            color_scc.r as f32,
            color_scc.g as f32,
            color_scc.b as f32,
            color_scc.a as f32,
        ]),

        Err(error) => {
            println!("parse_colour: {}", error);
            None
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
    buildings_or_parts: &mut Vec<BuildingOrPart>,
    nodes_map: &mut HashMap<u64, OsmNode>,
) {
    // println!("element = {:?}", element);
    //let tags_option = element.tags.unwrap(); // JosnTags { ..default() }; //ttt

    if element.tags.is_none() {
        println!("way without tags! ID {}", element.id);
        return;
    }

    let tags = element.tags.unwrap(); // JosnTags { ..default() }; //ttt
    let part = tags.building_part.unwrap_or(NO.to_string());
    // let name = tags.name.unwrap_or("-/-".to_string());
    // println!(" Way: building = {:?}  name = {:?}" name,);
    if part != YES {
        return; // ??? not only parts!
    };

    // Colors and Materials
    let color = parse_color(tags.color);
    let roof_color = parse_color(tags.roof_color);
    // println!("colors: {:?} {:?}", color, roof_color);

    // Heights
    let min_height = parse_height(tags.min_height);
    let mut part_height = parse_height(tags.height);
    let roof_height = parse_height(tags.roof_height);
    if roof_height.is_some() {
        part_height = Some(part_height.unwrap() - roof_height.unwrap());
    }
    let roof_shape = tags.roof_shape;
    let shape: RoofShape = match roof_shape {
        None => RoofShape::None,
        Some(shape) => match shape.as_str() {
            "flat" => RoofShape::Flat,
            "onion" => RoofShape::Onion,
            "pyramidal" => RoofShape::Phyramidal,
            _ => {
                println!("roof_shape Unknown: {}", shape);
                RoofShape::Unknown
            }
        },
    };

    // Get building footprint from nodes
    let nodes = element.nodes.unwrap();

    let mut north = 0.;
    let mut east = 0.;
    let mut foodprint: Vec<GroundPosition> = Vec::new();
    for node_id in nodes.iter().rev() {
        let node = nodes_map.get(node_id).unwrap();
        east += node.position.east;
        north += node.position.north;
        foodprint.push(node.position);
    }
    let count = nodes.len() as f32;
    north /= count;
    east /= count;
    let center = GroundPosition { north, east };
    println!("roof_shape: {:?}", shape);
    let roof = Roof {
        shape,
        height: roof_height,
        color: roof_color,
    };
    let building_or_part = BuildingOrPart {
        _part: part != NO, // ??? not only parts!
        foodprint,
        _center: center,
        height: part_height,
        min_height,
        roof: Some(roof),
        color,
    };

    buildings_or_parts.push(building_or_part);
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
) -> Vec<BuildingOrPart> {
    let mut buildings_or_parts: Vec<BuildingOrPart> = Vec::new();
    let mut nodes_map = HashMap::new();

    for element in json_data.elements {
        // println!("element.element_type: {}", element.element_type);
        match element.element_type.as_str() {
            "node" => node(element, ground_null_coordinates, &mut nodes_map),
            "way" => way(element, &mut buildings_or_parts, &mut nodes_map),
            _ => println!("Unknown element type: {}", element.element_type),
        }
    }

    buildings_or_parts
}
