use std::collections::HashMap;
//use std::fmt::Error;

use openstreetmap_api::Openstreetmap;
use openstreetmap_api::errors::OpenstreetmapError;
use openstreetmap_api::types::{Credentials, Node, Way};

use crate::kernel_in::{BoundingBox, BuildingPart, GeographicCoordinates, GroundPosition, OsmNode};
use crate::osm2layers::building;
use crate::shape::Shape;

///////////////////////////////////////////////////////////////////////////////////////////////////
// openstreetmap_api //////////////////////////////////////////////////////////////////////////////

static YES: &str = "yes";
static NO: &str = "no";

// DONT USE?:  https://api.openstreetmap.org/api/0.6/way/121486088/full.json
// https://master.apis.dev.openstreetmap.org/api/0.6/way/121486088/full.json
// The test-server does not have needed objects (like Reifenberg), but they could be PUT into
// static API_URL: &str = "https://api.openstreetmap.org/api/0.6/";

//#[derive(Default)]
pub struct InputLib {
    client: Openstreetmap,
}

impl Default for Shape {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for InputLib {
    fn default() -> Self {
        Self::new()
    }
}

impl InputLib {
    pub fn new() -> Self {
        let host = "https://api.openstreetmap.org/api".to_string(); // env::var("OPENSTREETMAP_HOST")?;
        let client = Openstreetmap::new(host, Credentials::None);
        Self { client }
    }

    pub async fn geo_bbox_of_way(&self, way_id: u64) -> Result<BoundingBox, OpenstreetmapError> {
        let way = self.client.ways().full(way_id).await?;

        let mut bounding_box = BoundingBox::new();
        // add the coordinates of all nodes
        for node in way.nodes {
            bounding_box.include(&GroundPosition {
                north: node.lat.unwrap() as f32,
                east: node.lon.unwrap() as f32,
            });
        }
        Ok(bounding_box)
    }

    pub async fn scan_osm(
        &self,
        bounding_box: &BoundingBox,
        gpu_ground_null_coordinates: &GeographicCoordinates,
        show_only: u64,
    ) -> Result<Vec<BuildingPart>, OpenstreetmapError> {
        let bounding_box = openstreetmap_api::types::BoundingBox {
            left: bounding_box.west as f64,
            bottom: bounding_box.south as f64,
            right: bounding_box.east as f64,
            top: bounding_box.north as f64,
        };
        let map = self.client.map(&bounding_box).await?;

        let mut building_parts: Vec<BuildingPart> = Vec::new();
        let mut nodes_map = HashMap::new();

        for node_ in map.nodes {
            node(node_, gpu_ground_null_coordinates, &mut nodes_map);
        }

        for way_ in map.ways {
            way(way_, &mut building_parts, &mut nodes_map, show_only);
        }

        Ok(building_parts)
    }
}

fn node(
    element: Node,
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
    mut element: Way,
    building_parts: &mut Vec<BuildingPart>,
    nodes_map: &mut HashMap<u64, OsmNode>,
    show_only_this_id: u64,
) {
    // println!("element = {:?}", element);
    if show_only_this_id > 0 && element.id != show_only_this_id {
        return;
    } // tttt

    if element.tags.is_empty() {
        // ttt println!( "way without tags! ID: {} Relation(-Outer) or Multipolligon?",element.id);
        return;
    }

    let string_no = &NO.to_string();
    let mut tags: HashMap<String, String> = HashMap::new();
    for tag in &element.tags {
        tags.insert(tag.k.clone(), tag.v.clone());
    }

    let part = tags.get("building:part").unwrap_or(string_no);
    let id = element.id;

    // Validate way-nodes
    let nodes = &mut element.node_refs;
    if nodes.len() < 3 {
        println!("Building with < 3 corners! id: {}", element.id);
        return;
    }
    if nodes.first().unwrap() != nodes.last().unwrap() {
        println!("Building with < 3 corners! id: {}", element.id);
        return;
    } else {
        nodes.pop();
    }

    let mut footprint = Shape::new();
    for node_ref in element.node_refs.iter() {
        let node = nodes_map.get(&node_ref.node_id).unwrap();
        footprint.push(node.position);
    } // nodes
    footprint.close();

    // ??? not only parts!
    if part == YES || show_only_this_id > 0 {
        building(footprint, id, &tags, building_parts);
    }
}
