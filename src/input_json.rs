use serde::Deserialize;
use std::collections::HashMap;

use crate::kernel_in::{BoundingBox, BuildingPart, GeographicCoordinates, OsmNode, RoofShape};
use crate::shape::Shape;
use crate::tagticks::{
    circle_limit, parse_color, parse_height, DEFAULT_ROOF_COLOR, DEFAULT_ROOF_HEIGHT,
    DEFAULT_WALL_COLOR, DEFAULT_WALL_HEIGHT,
};

///////////////////////////////////////////////////////////////////////////////////////////////////
// JOSN ///////////////////////////////////////////////////////////////////////////////////////////

static YES: &str = "yes";
static NO: &str = "no";

/* Factor to calculate meters from gps coordiantes.decimals (latitude, Nort/South position) */

// todo: &str   https://users.rust-lang.org/t/requires-that-de-must-outlive-static-issue/91344/10
#[derive(Deserialize, Debug)]
struct JosnElement {
    #[serde(rename = "type")]
    element_type: String,
    id: u64,
    lat: Option<f64>,
    lon: Option<f64>,
    nodes: Option<Vec<u64>>,
    //tags: Option<JosnTags>, // todo?: use a map
    tags: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug)]
pub struct JsonData {
    elements: Vec<JosnElement>,
}

static API_URL: &str = "https://api.openstreetmap.org/api/0.6/";

pub fn get_way_json(way_id: i64) -> JsonData {
    //// Get OSM data from API and convert Json to Rust types. See https://serde.rs
    let url = format!("{}way/{}/full.json", API_URL, way_id);
    reqwest::blocking::get(url).unwrap().json().unwrap()
}

pub fn get_range_json(bounding_box: BoundingBox) -> JsonData {
    // https://wiki.openstreetmap.org/wiki/API_v0.6#Retrieving_map_data_by_bounding_box:_GET_/api/0.6/map
    // GET   /api/0.6/map?bbox=left,bottom,right,top
    let url = format!("{}map.json?bbox={}", API_URL, bounding_box.to_string());
    reqwest::blocking::get(url).unwrap().json().unwrap()
}

// This is an extra fn to start the App. It should be possilbe to use one of the "normal" fu s?
pub fn coordinates_of_way_center(way_id: i64) -> GeographicCoordinates {
    // DONT USE?:  https://api.openstreetmap.org/api/0.6/way/121486088/full.json
    // https://master.apis.dev.openstreetmap.org/api/0.6/way/121486088/full.json
    // The test-server does not have needed objects (like Reifenberg), but they could be PUT into

    let json_way: JsonData = get_way_json(way_id);

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

pub fn scan_json(
    json_data: JsonData,
    ground_null_coordinates: &GeographicCoordinates,
    show_only: u64,
) -> Vec<BuildingPart> {
    let mut building_parts: Vec<BuildingPart> = Vec::new();
    let mut nodes_map = HashMap::new();

    for element in json_data.elements {
        // println!("element.element_type: {}", element.element_type);
        match element.element_type.as_str() {
            "node" => node(element, ground_null_coordinates, &mut nodes_map),
            "way" => way(element, &mut building_parts, &mut nodes_map, show_only),
            _ => (), //println!(  todo
                     //    "Error: Unknown element type: {}  id: {}",
                     //    element.element_type, element.id
                     //),
        }
    }

    building_parts
}

fn node(
    element: JosnElement,
    ground_null_coordinates: &GeographicCoordinates,
    nodes_map: &mut HashMap<u64, OsmNode>,
) {
    let osm_node = OsmNode {
        position: ground_null_coordinates
            .coordinates_to_position(element.lat.unwrap(), element.lon.unwrap()),
    };
    nodes_map.insert(element.id, osm_node);
    // println!("Node: id = {:?} lat = {:?} lon = {:?}", element.id, element.lat.unwrap(), element.lon.unwrap() );
}

fn way(
    element: JosnElement,
    building_parts: &mut Vec<BuildingPart>,
    nodes_map: &mut HashMap<u64, OsmNode>,
    show_only_this_id: u64,
) {
    // println!("element = {:?}", element);

    if show_only_this_id > 0 && element.id != show_only_this_id {
        return;
    } // tttt

    if element.tags.is_none() {
        // ttt println!( "way without tags! ID: {} Relation(-Outer) or Multipolligon?",element.id);
        return;
    }

    let string_no = &NO.to_string();
    let tags = element.tags.as_ref().unwrap();
    let part = tags.get("building:part").unwrap_or(string_no);

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
    // Validate way-nodes
    let mut nodes = element.nodes.unwrap();
    if nodes.len() < 3 {
        println!("Building with < 3 corners! id: {}", element.id);
        return;
    }
    if nodes.first().unwrap() != nodes.last().unwrap() {
        println!("Building with < 3 corners! id: {}", element.id);
    } else {
        nodes.pop();
    }

    let tags = element.tags.unwrap();

    // Colors and Materials
    let color = parse_color(
        tags.get("colour")
            .unwrap_or(&DEFAULT_WALL_COLOR.to_string()),
    );
    let roof_color = parse_color(
        tags.get("roof:colour")
            .unwrap_or(&DEFAULT_ROOF_COLOR.to_string()),
    );

    // Shape of the roof. All buildings have a roof, even if it is not tagged
    let roof_shape: RoofShape = match tags.get("roof:shape") {
        Some(roof_shape) => match roof_shape.as_str() {
            "flat" => RoofShape::Flat,
            "skillion" => RoofShape::Skillion,
            "onion" => RoofShape::Onion,
            "pyramidal" => RoofShape::Phyramidal,
            _ => {
                //ttt println!("Warning: roof_shape Unknown: {}", roof_shape);
                RoofShape::Flat // todo: gabled and geographic dependend
            }
        },
        None => RoofShape::None,
    };

    println!("Part id: {} roof: {:?}", element.id, roof_shape);

    let default_roof_heigt = match roof_shape {
        RoofShape::Skillion => 9.0, // 2.?  accroding to width! ttt
        _ => DEFAULT_ROOF_HEIGHT,
    };

    // Heights
    let min_height = parse_height(tags.get("min_height"), 0.0);
    let roof_height = parse_height(tags.get("roof:height"), default_roof_heigt);
    let wall_height = parse_height(tags.get("height"), DEFAULT_WALL_HEIGHT) - roof_height;

    // Get building footprint from nodes
    // else { todo("drop last and modulo index") }

    //let mut footprint: Vec<GroundPosition> = Vec::new();
    let mut footprint = Shape::new();
    for node_id in nodes.iter() {
        let node = nodes_map.get(node_id).unwrap();
        footprint.push(node.position);
    } // nodes
    footprint.close();
    let mut roof_angle = footprint.longest_angle;

    //println!("ttt roof_angle: {}", roof_angle.to_degrees());

    // todo: more angle code!
    let bounding_box_rotated = footprint.rotate(-roof_angle);
    //println!("bbox________ {:?}", footprint.bounding_box);
    //println!("bbox_rotated {:?}", bounding_box_rotated);
    if bounding_box_rotated.east_larger_than_nord() {
        roof_angle = circle_limit(roof_angle + f32::to_radians(90.));
        // This way is a good example: 363815745 beause it has many nodes on the longer side
        // println!( "### {}: east_larger_than_nord: {}", element.id, roof_angle.to_degrees() );
    }

    //println!(
    //    "id: {} roof_shape: {:?} angle: {}",
    //    element.id,
    //    roof_shape,
    //    roof_angle.to_degrees()
    //);
    let building_part = BuildingPart {
        _id: element.id,
        _part: true, // ??? not only parts!
        footprint,
        //center,
        // _bounding_box: bounding_box,
        bounding_box_rotated,
        wall_height,
        min_height,
        color,
        roof_shape,
        roof_height,
        roof_angle,
        roof_color,
    };

    building_parts.push(building_part);
}
