use bytes::*;
use serde::Deserialize;
use std::collections::HashMap;

use crate::kernel_in::{
    BoundingBox, BuildingsOrParts, GeographicCoordinates, GroundPosition, Member,
};
use crate::osm2layers::Osm2Layer;

const LOCAL_TEST: bool = false;

///////////////////////////////////////////////////////////////////////////////////////////////////
// JOSN ///////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct InputOsm {
    api_url: String,
    //
}

impl Default for InputOsm {
    fn default() -> Self {
        Self::new()
    }
}

impl InputOsm {
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
        let mut url = format!("{}way/{}/full.json", self.api_url, way_id);
        if LOCAL_TEST {
            url = "bbox.json".to_string();
        }
        println!("= Way_URL: {url}");
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
    ) -> Result<BuildingsOrParts, Box<dyn std::error::Error>> {
        let mut url = format!("{}map.json?bbox={}", self.api_url, bounding_box);
        if LOCAL_TEST {
            url = "way.json".to_string();
        }
        println!("= BBox_URL: {url}");
        let bytes = reqwest::get(url).await?.bytes().await?;
        Ok(scan_json_to_osm_bytes(
            bytes,
            gpu_ground_null_coordinates,
            show_only,
        ))
    }

    pub fn scan_json_to_osm_vec(
        &self,
        bytes: &[u8],
        gpu_ground_null_coordinates: &GeographicCoordinates,
        show_only: u64,
    ) -> BuildingsOrParts {
        let json_bbox_data: JsonData = serde_json::from_slice(bytes).unwrap();
        scan_json_to_osm(json_bbox_data, gpu_ground_null_coordinates, show_only)
    }
}

static API_URL: &str = "https://api.openstreetmap.org/api/0.6/";

pub fn bbox_url(bounding_box: &BoundingBox) -> String {
    // https://wiki.openstreetmap.org/wiki/API_v0.6#Retrieving_map_data_by_bounding_box:_GET_/api/0.6/map
    // GET   /api/0.6/map?bbox=left,bottom,right,top
    format!("{}map.json?bbox={}", API_URL, bounding_box)
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

pub fn scan_json_to_osm_bytes(
    bytes: Bytes,
    gpu_ground_null_coordinates: &GeographicCoordinates,
    show_only: u64,
) -> BuildingsOrParts {
    let json_bbox_data: JsonData = serde_json::from_slice(&bytes).unwrap();
    scan_json_to_osm(json_bbox_data, gpu_ground_null_coordinates, show_only)
}

pub fn scan_json_to_osm(
    json_bbox_data: JsonData,
    gpu_ground_null_coordinates: &GeographicCoordinates,
    show_only: u64,
) -> BuildingsOrParts {
    let mut osm2layer = Osm2Layer::create(*gpu_ground_null_coordinates, show_only);
    for element in json_bbox_data.elements {
        // println!("id: {}  type: {}", element.id, element.element_type);
        match element.element_type.as_str() {
            "node" => {
                osm2layer.add_node(element.id, element.lat.unwrap(), element.lon.unwrap(), None)
            }

            "way" => osm2layer.add_way(element.id, element.nodes.unwrap(), element.tags),

            "relation" => {
                osm2layer.add_relation(element.id, element.members.unwrap(), element.tags)
            }

            _ => println!(
                "Error: Unknown element type: {}  id: {}",
                element.element_type, element.id
            ),
        }
    }

    osm2layer.process_elements_from_osm_to_layers();

    osm2layer.get_building_parts()
}
