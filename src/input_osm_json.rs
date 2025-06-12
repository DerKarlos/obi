use bytes::*;
use serde::Deserialize;
use std::collections::HashMap;

//
//

use crate::kernel_in::{BoundingBox, BuildingPart, GeographicCoordinates, GroundPosition, OsmNode};
use crate::osm2layers::{building, tags_get2};
use crate::shape::Shape;

///////////////////////////////////////////////////////////////////////////////////////////////////
// JOSN ///////////////////////////////////////////////////////////////////////////////////////////

static _YES: &str = "yes";
static NO: &str = "no";

#[derive(Debug)]
pub struct InputJson {
    api_url: String, // just a dummy?
}

impl Default for InputJson {
    fn default() -> Self {
        Self::new()
    }
}

impl InputJson {
    pub fn new() -> Self {
        let api_url = "https://api.openstreetmap.org/api/0.6/".to_string();
        Self { api_url }
    }

    pub fn way_url(&self, way_id: u64) -> String {
        format!("{}way/{}/full.json", self.api_url, way_id)
    }

    pub fn bbox_url(&self, bounding_box: &BoundingBox) -> String {
        // https://wiki.openstreetmap.org/wiki/API_v0.6#Retrieving_map_data_by_bounding_box:_GET_/api/0.6/map
        // GET   /api/0.6/map?bbox=left,bottom,right,top
        format!("{}map.json?bbox={}", self.api_url, bounding_box)
    }

    pub async fn geo_bbox_of_way(
        &self,
        way_id: u64,
    ) -> Result<BoundingBox, Box<dyn std::error::Error>> {
        let url = format!("{}way/{}/full.json", self.api_url, way_id);
        println!("++++++++++ Way_URL: {url}");
        let bytes = reqwest::get(url).await?.bytes().await?;
        Ok(geo_bbox_of_way_bytes(&bytes))
    }

    pub fn geo_bbox_of_way_vec(&self, bytes: &[u8]) -> BoundingBox {
        let json_way_data: JsonData = serde_json::from_slice(bytes).unwrap();
        geo_bbox_of_way_json(json_way_data)
    }

    pub async fn scan_osm(
        &self,
        bounding_box: &BoundingBox,
        gpu_ground_null_coordinates: &GeographicCoordinates,
        show_only: u64,
    ) -> Result<Vec<BuildingPart>, Box<dyn std::error::Error>> {
        let url = format!("{}map.json?bbox={}", self.api_url, bounding_box);
        println!("++++++++++ BBox_URL: {url}");
        let bytes = reqwest::get(url).await?.bytes().await?;
        Ok(scan_osm_bytes(
            bytes,
            &gpu_ground_null_coordinates,
            show_only,
        ))
    }

    pub fn scan_osm_vec(
        &self,
        bytes: &[u8],
        gpu_ground_null_coordinates: &GeographicCoordinates,
        show_only: u64,
    ) -> Vec<BuildingPart> {
        let json_bbox_data: JsonData = serde_json::from_slice(bytes).unwrap();
        scan_osm_json(json_bbox_data, gpu_ground_null_coordinates, show_only)
    }
}

static API_URL: &str = "https://api.openstreetmap.org/api/0.6/";

pub fn bbox_url(bounding_box: &BoundingBox) -> String {
    // https://wiki.openstreetmap.org/wiki/API_v0.6#Retrieving_map_data_by_bounding_box:_GET_/api/0.6/map
    // GET   /api/0.6/map?bbox=left,bottom,right,top
    format!("{}map.json?bbox={}", API_URL, bounding_box)
}

#[derive(Deserialize, Debug, Clone)]
pub struct Member {
    #[serde(rename = "type")]
    element_type: String,
    #[serde(rename = "ref")]
    element_ref: u64,
    role: String,
}

// todo: &str   https://users.rust-lang.org/t/requires-that-de-must-outlive-static-issue/91344/10
#[derive(Deserialize, Debug, Clone)]
pub struct JosnElement {
    id: u64,
    #[serde(rename = "type")]
    element_type: String,
    lat: Option<f64>,
    lon: Option<f64>,
    nodes: Option<Vec<u64>>,
    members: Option<Vec<Member>>,
    tags: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug)]
pub struct JsonData {
    pub elements: Vec<JosnElement>,
}

pub fn geo_bbox_of_way_string(bytes: &&str) -> BoundingBox {
    let json_way_data: JsonData = serde_json::from_str(bytes).unwrap();
    geo_bbox_of_way_json(json_way_data)
}

pub fn geo_bbox_of_way_bytes(bytes: &Bytes) -> BoundingBox {
    let json_way_data: JsonData = serde_json::from_slice(bytes).unwrap();
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

pub fn scan_osm_bytes(
    bytes: Bytes,
    gpu_ground_null_coordinates: &GeographicCoordinates,
    show_only: u64,
) -> Vec<BuildingPart> {
    let json_bbox_data: JsonData = serde_json::from_slice(&bytes).unwrap();
    scan_osm_json(json_bbox_data, gpu_ground_null_coordinates, show_only)
}

pub fn scan_osm_json(
    json_bbox_data: JsonData,
    gpu_ground_null_coordinates: &GeographicCoordinates,
    show_only: u64,
) -> Vec<BuildingPart> {
    let mut building_parts: Vec<BuildingPart> = Vec::new();
    let mut nodes_map = HashMap::new();
    let mut ways_map = HashMap::new();

    for element in json_bbox_data.elements {
        // println!("element.element_type: {}", element.element_type);
        match element.element_type.as_str() {
            "node" => node(element, gpu_ground_null_coordinates, &mut nodes_map),
            "way" => way(
                element,
                &mut building_parts,
                &mut nodes_map,
                &mut ways_map,
                show_only,
            ),
            "relation" => relation(element, &mut building_parts, &mut ways_map, show_only),
            _ => println!(
                "Error: Unknown element type: {}  id: {}",
                element.element_type, element.id
            ),
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
    ways_map: &mut HashMap<u64, Shape>,
    show_only: u64,
) {
    // Validate way-nodes

    let nodes = &mut element.nodes.unwrap();

    // Only closed ways (yet)
    if nodes.first().unwrap() != nodes.last().unwrap() {
        //println!("Not a closed way id: {}", element.id);
        return;
    }

    if nodes.len() < 3 {
        println!("Closed way with < 3 corners! id: {}", &element.id);
        return;
    } else {
        nodes.pop();
    }

    // println!("element = {:?}", element);

    let mut footprint = Shape::new();
    for node_id in nodes.iter() {
        let node = nodes_map.get(node_id).unwrap();
        footprint.push(node.position);
    } // nodes
    footprint.close();
    ways_map.insert(element.id, footprint.clone());

    if show_only > 0 && element.id != show_only {
        return;
    }

    if element.tags.is_none() {
        return;
    }

    let string_no = &NO.to_string();
    let tags = element.tags.as_ref().unwrap();
    let part = tags.get("building:part").unwrap_or(string_no);
    let id = element.id;

    // ??? not only parts!    || show_only < 0
    if part != NO || show_only > 0 {
        building(footprint, id, tags, building_parts);
    }
}

fn relation(
    element: JosnElement,
    building_parts: &mut Vec<BuildingPart>,
    mut ways_map: &mut HashMap<u64, Shape>,
    show_only: u64,
) {
    // https://api.openstreetmap.org/api/0.6/relation/8765346/full.json
    //println!("relation, id: {:?} {}", element.id, show_only);

    if show_only > 0 && element.id != show_only {
        // show_only
        return;
    }

    println!("Relation, id: {:?}", element.id);

    if element.members.is_none() {
        println!("Relation without members! id: {:?}", element.id);
        return;
    }

    let members = element.members.unwrap();

    let tags = &element.tags.unwrap();
    let relation_type = tags.get("type").unwrap();
    if relation_type != "multipolygon" {
        //println!("Unprocessed relation type: {relation_type}");
        return;
    }

    //println!("rel tags: {:?}", tags);
    let part_option = tags_get2(tags, "building:part", "building");
    if part_option.is_none() && show_only == 0 {
        //println!("Unprocessed relation non-part tag {}", element.id);
        return;
    }

    let mut footprint = Shape::new();

    for member in members {
        //println!("mem: {:?}", &member);
        if member.element_type != "way" {
            return;
        }
        match member.role.as_str() {
            "outer" => {
                let outer_ref = member.element_ref;
                let option = ways_map.get(&outer_ref);
                if option.is_none() {
                    println!("outer none, id/ref: {}", outer_ref);
                    return;
                }
                footprint = ways_map.get(&outer_ref).unwrap().clone();
            }
            "inner" => {
                inner(&mut ways_map, member.element_ref, &mut footprint);
            }
            _ => (),
        }
    }

    let id = element.id;
    building(footprint, id, tags, building_parts);
}

fn inner(ways_map: &mut HashMap<u64, Shape>, elements_ref: u64, footprint: &mut Shape) {
    //println!("elements_ref: {:?}", &elements_ref);
    let option = ways_map.get(&elements_ref);
    if option.is_none() {
        println!("inner none, id/ref: {}", elements_ref);
        return;
    }
    let hole = ways_map.get(&elements_ref).unwrap().clone();
    footprint.push_hole(hole);
    //println!("outer_way; {:?}", &outer_way);
}
