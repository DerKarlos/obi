use bytes::*;
use serde::Deserialize;

use crate::kernel_in::{
    BoundingBox, BuildingsAndParts, FGP, GeographicCoordinates, GroundPosition, Members, OsmMap,
};
use crate::osm2layers::{Osm2Layer, tags_get_yes};

const LOCAL_TEST: bool = false;

///////////////////////////////////////////////////////////////////////////////////////////////////
// JOSN ///////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct InputOsm {
    api_url: String,
}

impl Default for InputOsm {
    fn default() -> Self {
        Self::new()
    }
}

impl InputOsm {
    pub fn new() -> Self {
        let api_url = "https://api.openstreetmap.org/api/0.6/".into();
        Self { api_url }
    }

    pub fn element_url(&self, way_id: u64, is_way: bool) -> String {
        let way_or_relation = if is_way { "way" } else { "relation" };
        format!("{}{}/{}/full.json", self.api_url, way_or_relation, way_id)
    }

    pub fn bbox_url(&self, bounding_box: &BoundingBox) -> String {
        // https://wiki.openstreetmap.org/wiki/API_v0.6#Retrieving_map_data_by_bounding_box:_GET_/api/0.6/map
        // GET   /api/0.6/map?bbox=left,bottom,right,top
        format!("{}map.json?bbox={}", self.api_url, bounding_box)
    }

    pub async fn geo_bbox_of_element(
        &self,
        way_id: u64,
        is_way: bool,
    ) -> Result<BoundingBox, Box<dyn std::error::Error>> {
        let mut url = self.element_url(way_id, is_way); // format!("{}way/{}/full.json", self.api_url, way_id);
        if LOCAL_TEST {
            url = "bbox.json".into();
        }

        #[cfg(debug_assertions)]
        println!("= Way_URL: {url}");

        let response = reqwest::get(url).await?;
        match response.status().as_u16() {
            200 => (),
            404 => println!("Way {} does not exist (404)", way_id),
            410 => println!("Way {} is deleted (410)", way_id),
            _ => panic!("Way read error {:?}", response.status().as_u16()),
        }
        let result = response.bytes().await; //e?;
        //println!("result1: {:?}", result);

        match result {
            // this code is messy, isnt it ??? replace by crate geo
            Ok(bytes) => {
                let option = geo_bbox_of_way_bytes(&bytes, way_id, is_way);
                if let Some(bounding_box) = option {
                    Ok(bounding_box)
                } else {
                    Ok(BoundingBox::ZERO)
                }
            }
            Err(e) => {
                println!("Way bytes Loading Error: {}", e);
                Ok(BoundingBox::ZERO)
            }
        }

        // Ok(geo_bbox_of_way_bytes(&bytes))
    }

    pub fn geo_bbox_of_element_vec(&self, bytes: &[u8], id: u64, is_way: bool) -> BoundingBox {
        let json_way_data: JsonData = serde_json::from_slice(bytes).unwrap();
        geo_bbox_of_element_json(json_way_data, id, is_way)
    }

    pub async fn scan_osm(
        &self,
        bounding_box: &BoundingBox,
        gpu_ground_null_coordinates: &GeographicCoordinates,
        show_only: u64,
        way_only: u64,
    ) -> Result<BuildingsAndParts, Box<dyn std::error::Error>> {
        let mut url = format!("{}map.json?bbox={}", self.api_url, bounding_box);
        if LOCAL_TEST {
            url = "way.json".into();
        }

        #[cfg(debug_assertions)]
        println!("= BBox_URL: {url}");

        // let bytes = reqwest::get(url).await?.bytes().await?;
        let response = reqwest::get(url).await?;
        match response.status().as_u16() {
            200 => (),
            400 => println!("Bad Request: Map limits are exceeded (400)"),
            509 => println!("Bandwidth Limit Exceeded: too much data downloaded (509)"),
            _ => panic!("Load Map Error: {:?}", response.status().as_u16()),
        }
        let bytes = response.bytes().await;

        match bytes {
            Ok(bytes) => Ok(scan_json_bytes_to_osm(
                bytes,
                gpu_ground_null_coordinates,
                show_only,
                way_only,
            )),
            Err(e) => {
                println!("Area bytes Loading Error: {}", e);
                panic!("Area bytes Loading Error: {:?}", e);
                //Error(e)
            }
        }
    }

    pub fn scan_json_to_osm_vec(
        &self,
        bytes: &[u8],
        gpu_ground_null_coordinates: &GeographicCoordinates,
        show_only: u64,
        way_only: u64,
    ) -> BuildingsAndParts {
        let json_bbox_data: JsonData = serde_json::from_slice(bytes).unwrap();
        scan_json_to_osm(
            json_bbox_data,
            gpu_ground_null_coordinates,
            show_only,
            way_only,
        )
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
    members: Option<Members>,
    tags: Option<OsmMap>,
}

#[derive(Deserialize, Debug)]
pub struct JsonData {
    pub elements: Vec<JosnElement>,
}

pub fn geo_bbox_of_way_string(bytes: &&str, way_id: u64, is_way: bool) -> BoundingBox {
    let json_way_data: JsonData = serde_json::from_str(bytes).unwrap();
    geo_bbox_of_element_json(json_way_data, way_id, is_way)
}

pub fn geo_bbox_of_way_bytes(bytes: &Bytes, way_id: u64, is_way: bool) -> Option<BoundingBox> {
    let result = serde_json::from_slice(bytes);
    match result {
        Ok(json_way_data) => Some(geo_bbox_of_element_json(json_way_data, way_id, is_way)),
        Err(_e) => None,
    }
}

// This is an extra fn to start the App. It should be possilbe to use one of the "normal" fu s?
pub fn geo_bbox_of_element_json(
    json_way_data: JsonData,
    element_id: u64,
    is_way: bool,
) -> BoundingBox {
    //let json_way: JsonData = get_way_json(way_id).await;
    //let json_way = get_way_json(way_id).await.unwrap();

    // println!("Received JSON: {}", json_way),
    let mut bounding_box = BoundingBox::new();
    // add the coordinates of all nodes
    for element in json_way_data.elements {
        if (element.element_type == "way" && element.id == element_id && is_way)
            || (element.element_type == "relation" && element.id == element_id && !is_way)
        {
            if let Some(tags) = element.tags {
                if tags_get_yes(&tags, "building:part").is_some() {
                    println!("Inspected Way is not a building but a part!");
                    return BoundingBox::ZERO;
                }
            }
        }

        if element.element_type == "node" {
            bounding_box.include(&GroundPosition {
                north: element.lat.unwrap() as FGP,
                east: element.lon.unwrap() as FGP,
            });
        }
    }
    bounding_box
}

pub fn scan_json_bytes_to_osm(
    bytes: Bytes,
    gpu_ground_null_coordinates: &GeographicCoordinates,
    show_only: u64,
    way_only: u64,
) -> BuildingsAndParts {
    let result = serde_json::from_slice(&bytes);
    //println!("result: {:?}", result);
    if result.is_err() {
        return Vec::new();
    }
    let json_bbox_data: JsonData = result.unwrap();
    scan_json_to_osm(
        json_bbox_data,
        gpu_ground_null_coordinates,
        show_only,
        way_only,
    )
}

pub fn scan_json_to_osm(
    json_bbox_data: JsonData,
    gpu_ground_null_coordinates: &GeographicCoordinates,
    show_only: u64,
    way_only: u64,
) -> BuildingsAndParts {
    let mut osm2layer = Osm2Layer::create(*gpu_ground_null_coordinates, show_only, way_only);
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

    osm2layer.process_elements();

    osm2layer.get_buildings_and_parts()
}
