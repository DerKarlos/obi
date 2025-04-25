use std::collections::HashMap;

use csscolorparser::parse;
use serde::Deserialize;

//e triangulate::{self, formats, Polygon};
//use csscolorparser::parse;
//use triangulation::{Delaunay, Point};

use crate::input_api::{
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
    roof_colour: Option<String>,
    colour: Option<String>,
    #[serde(rename = "roof:height")]
    roof_height: Option<String>,
    height: Option<String>,
    min_height: Option<String>,
}

#[derive(Deserialize, Debug)]
struct JsonData {
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

fn parse_color(colour: Option<String>) -> Option<RenderColor> {
    //if
    colour.as_ref()?;
    //    is_none() {
    //    return None;
    //};

    match parse(colour.unwrap().as_str()) {
        Ok(colour_scc) => Some([
            colour_scc.r as f32,
            colour_scc.g as f32,
            colour_scc.b as f32,
            colour_scc.a as f32,
        ]),

        Err(error) => {
            println!("parse_colour: {}", error);
            None
        }
    }
}

fn to_position(coordiantes: &GeographicCoordinates, lat: f64, lon: f64) -> GroundPosition {
    // the closer to the pole, the smaller the tiles size in meters get
    let lon_fakt = LAT_FAKT * ((lat / 180. * PI as f64).abs()).cos(); // Longitude(Längengrad) West/East factor
                                                                      // actual coor - other coor = relative grad/meter ground position
    let east = ((lon - coordiantes.longitude) * lon_fakt) as f32;
    let north = ((lat - coordiantes.latitude) * LAT_FAKT) as f32;
    /*return*/
    GroundPosition { east, north }
}

pub fn coordinates_of_way(way_id: u64) -> GeographicCoordinates {
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
    let color = parse_color(tags.colour);
    let roof_color = parse_color(tags.roof_colour);
    // println!("colors: {:?} {:?}", colour, roof_colour);

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

    let mut foodprint: Vec<GroundPosition> = Vec::new();
    for node_id in nodes.iter().rev() {
        let node = nodes_map.get(node_id).unwrap();
        foodprint.push(node.position);
    }

    println!("roof_shape: {:?}", shape);
    let roof = Roof {
        shape,
        height: roof_height,
        color: roof_color,
    };
    let building_or_part = BuildingOrPart {
        _part: true, // ??? not only parts!
        height: part_height,
        min_height,
        roof: Some(roof),
        foodprint,
        color,
    };

    buildings_or_parts.push(building_or_part);
}

pub fn scan_json(
    ground_null_coordinates: &GeographicCoordinates,
    range: f64,
) -> Vec<BuildingOrPart> {
    let mut buildings_or_parts: Vec<BuildingOrPart> = Vec::new();
    let mut nodes_map = HashMap::new();

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
    // range: x=4209900.5 z=-4290712 url=
    // https://api.openstreetmap.org/api/0.6/map.json?bbox=11.135635953165316,49.75577293983198,11.135905980168015,49.75604296683468
    // https://api.openstreetmap.org/api/0.6/map.json?bbox=76.36808519471933,64.41713173392363,76.75875957883649,64.50167155517451

    //t url = format!("https://api.openstreetmap.org/api/0.6/way/{}/full.json", way_id);
    let json_map: JsonData = reqwest::blocking::get(url).unwrap().json().unwrap();

    for element in json_map.elements {
        match element.element_type.as_str() {
            "node" => node(element, ground_null_coordinates, &mut nodes_map),
            "way" => way(element, &mut buildings_or_parts, &mut nodes_map),
            _ => println!("Unknown element type: {}", element.element_type),
        }
    }

    buildings_or_parts
}
