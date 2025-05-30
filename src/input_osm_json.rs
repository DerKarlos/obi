use bytes::*;
use serde::Deserialize;
use std::collections::HashMap;

use crate::kernel_in::{
    BoundingBox, BuildingPart, GeographicCoordinates, GroundPosition, OsmNode, RoofShape,
};
use crate::shape::Shape;
use crate::tagticks::{
    DEFAULT_ROOF_COLOR, DEFAULT_ROOF_HEIGHT, DEFAULT_WALL_COLOR, DEFAULT_WALL_HEIGHT, circle_limit,
    parse_color, parse_height,
};

///////////////////////////////////////////////////////////////////////////////////////////////////
// JOSN ///////////////////////////////////////////////////////////////////////////////////////////

static YES: &str = "yes";
static NO: &str = "no";

// DONT USE?:  https://api.openstreetmap.org/api/0.6/way/121486088/full.json
// https://master.apis.dev.openstreetmap.org/api/0.6/way/121486088/full.json
// The test-server does not have needed objects (like Reifenberg), but they could be PUT into
static API_URL: &str = "https://api.openstreetmap.org/api/0.6/";

#[derive(Default, Clone, Copy, Debug)]
pub struct OsmApi {
    _dummy: f32,
}

impl OsmApi {
    pub fn new() -> Self {
        Self { _dummy: 0. }
    }

    pub fn way_url(&self, way_id: u64) -> String {
        format!("{}way/{}/full.json", API_URL, way_id)
    }

    pub fn bbox_url(&self, bounding_box: &BoundingBox) -> String {
        // https://wiki.openstreetmap.org/wiki/API_v0.6#Retrieving_map_data_by_bounding_box:_GET_/api/0.6/map
        // GET   /api/0.6/map?bbox=left,bottom,right,top
        format!("{}map.json?bbox={}", API_URL, bounding_box.to_string())
    }
}

pub fn way_url(way_id: u64) -> String {
    format!("{}way/{}/full.json", API_URL, way_id)
}

pub fn bbox_url(bounding_box: &BoundingBox) -> String {
    // https://wiki.openstreetmap.org/wiki/API_v0.6#Retrieving_map_data_by_bounding_box:_GET_/api/0.6/map
    // GET   /api/0.6/map?bbox=left,bottom,right,top
    format!("{}map.json?bbox={}", API_URL, bounding_box.to_string())
}

// todo: &str   https://users.rust-lang.org/t/requires-that-de-must-outlive-static-issue/91344/10
#[derive(Deserialize, Debug)]
pub struct JosnElement {
    id: u64,
    #[serde(rename = "type")]
    element_type: String,
    lat: Option<f64>,
    lon: Option<f64>,
    nodes: Option<Vec<u64>>,
    tags: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug)]
pub struct JsonData {
    pub elements: Vec<JosnElement>,
}

pub fn geo_bbox_of_way_vec(bytes: &Vec<u8>) -> BoundingBox {
    let json_way_data: JsonData = serde_json::from_slice(&bytes).unwrap();
    geo_bbox_of_way_json(json_way_data)
}

pub fn geo_bbox_of_way_bytes(bytes: &Bytes) -> BoundingBox {
    let json_way_data: JsonData = serde_json::from_slice(&bytes).unwrap();
    geo_bbox_of_way_json(json_way_data)
}

// This is an extra fn to start the App. It should be possilbe to use one of the "normal" fu s?
pub fn geo_bbox_of_way_json(json_way_data: JsonData) -> BoundingBox {
    //let json_way: JsonData = get_way_json(way_id).await;

    //let json_way = get_way_json(way_id).await.unwrap();
    // println!("Received JSON: {}", json_way),
    let mut bounding_box = BoundingBox::new();
    // add the coordinates of all nodes
    for element in json_way_data.elements {
        if element.element_type == "node" {
            bounding_box.include(&GroundPosition {
                north: element.lat.unwrap() as f32,
                east: element.lon.unwrap() as f32,
            });
        }
    }
    bounding_box
}

pub fn scan_osm_vec(
    bytes: &Vec<u8>,
    gpu_ground_null_coordinates: &GeographicCoordinates,
    show_only: u64,
) -> Vec<BuildingPart> {
    let json_bbox_data: JsonData = serde_json::from_slice(&bytes).unwrap();
    scan_osm_json(json_bbox_data, &gpu_ground_null_coordinates, show_only)
}

pub fn scan_osm_bytes(
    bytes: Bytes,
    gpu_ground_null_coordinates: &GeographicCoordinates,
    show_only: u64,
) -> Vec<BuildingPart> {
    let json_bbox_data: JsonData = serde_json::from_slice(&bytes).unwrap();
    scan_osm_json(json_bbox_data, &gpu_ground_null_coordinates, show_only)
}

pub fn scan_osm_json(
    json_bbox_data: JsonData,
    gpu_ground_null_coordinates: &GeographicCoordinates,
    show_only: u64,
) -> Vec<BuildingPart> {
    let mut building_parts: Vec<BuildingPart> = Vec::new();
    let mut nodes_map = HashMap::new();

    for element in json_bbox_data.elements {
        // println!("element.element_type: {}", element.element_type);
        match element.element_type.as_str() {
            "node" => node(element, gpu_ground_null_coordinates, &mut nodes_map),
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
    gpu_ground_null_coordinates: &GeographicCoordinates,
    nodes_map: &mut HashMap<u64, OsmNode>,
) {
    let osm_node = OsmNode {
        position: gpu_ground_null_coordinates
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
    if part == YES || show_only_this_id > 0 {
        building(element, building_parts, nodes_map);
    }
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
            "gabled" => RoofShape::Gabled,
            _ => {
                //ttt println!("Warning: roof_shape Unknown: {}", roof_shape);
                RoofShape::Flat // todo: gabled and geographic dependend
            }
        },
        None => RoofShape::None,
    };

    println!("Part id: {} roof: {:?}", element.id, roof_shape);

    let default_roof_heigt = match roof_shape {
        RoofShape::Skillion => 2.0, // accroding to width???
        RoofShape::Gabled => 2.0,   // 2.?  ttt
        _ => DEFAULT_ROOF_HEIGHT,
    };

    // Heights
    let min_height = parse_height(tags.get("min_height"), 0.0);
    let roof_height = parse_height(tags.get("roof:height"), default_roof_heigt);
    //println!(        "roof_height: {roof_height}  {default_roof_heigt} {:?}",        roof_shape);
    let wall_height = parse_height(tags.get("height"), DEFAULT_WALL_HEIGHT) - roof_height;

    // Get building footprint from nodes
    // else { todo("drop last and modulo index") }

    // Roof direction and Orientation
    let mut footprint = Shape::new();
    for node_id in nodes.iter() {
        let node = nodes_map.get(node_id).unwrap();
        footprint.push(node.position);
    } // nodes
    footprint.close();

    // todo: parse_direction
    let mut roof_angle = footprint.longest_angle;
    let roof_orientation = tags.get("roof:orientation");
    // https://wiki.openstreetmap.org/wiki/Key:roof:orientation

    // Wwired: OSM defines the roof-angle value as across the lonest way side! So, ...
    if let Some(orientation) = roof_orientation {
        match orientation.as_str() {
            // ... the default along needs a rotation ...
            "along" => roof_angle = circle_limit(roof_angle + f32::to_radians(90.)),
            // ... while across is already given.
            "across" => (),
            _ => println!("Uncoded roof orientation value: {}", orientation),
        }
    } else {
        // ... the default along needs a rotation.
        roof_angle = circle_limit(roof_angle + f32::to_radians(90.));
    }

    let roof_direction = /*parse_orientation???*/ tags.get("roof:direction");
    if let Some(direction) = roof_direction {
        match direction.as_str() {
            "N" => roof_angle = f32::to_radians(0.),
            "E" => roof_angle = f32::to_radians(90.),
            "S" => roof_angle = f32::to_radians(180.), // todo: skilleon direction 90 different?!
            "W" => roof_angle = f32::to_radians(270.),

            "NE" => roof_angle = f32::to_radians(45.),
            "NW" => roof_angle = f32::to_radians(315.),
            "SE" => roof_angle = f32::to_radians(135.),
            "SW" => roof_angle = f32::to_radians(225.),
            _ => {
                let value = direction.parse();
                if let Ok(value) = value {
                    roof_angle = circle_limit(roof_angle + f32::to_radians(value));
                } else {
                    println!("Uncoded roof direction value: {}", direction);
                }
            }
        }
    }

    //println!("ttt roof_angle: {}", roof_angle.to_degrees());

    // This crate interprets, opposite to OSM the angle along the roof ceiling. Change this???
    roof_angle = circle_limit(roof_angle - f32::to_radians(90.));

    // Not here, in the fn rotate against the actual angle to got 0 degrees
    let bounding_box_rotated = footprint.rotate(roof_angle);

    // This seems NOT to be valid. F4maps is NOT doing it ??? Test with Reifenberg
    // if bounding_box_rotated.east_larger_than_nord() {
    //     roof_angle = circle_limit(roof_angle + f32::to_radians(90.));
    // This way is a good example: 363815745 beause it has many nodes on the longer side
    // println!( "### {}: east_larger_than_nord: {}", element.id, roof_angle.to_degrees() );
    //}
    // println!( "id: {} roof_shape: {:?} angle: {}", element.id, roof_shape, roof_angle.to_degrees() );

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
