//
//
use std::collections::HashMap;

use openstreetmap_api::Openstreetmap;
use openstreetmap_api::errors::OpenstreetmapError;
use openstreetmap_api::types::Credentials;

use crate::kernel_in::{
    BoundingBox, BuildingsAndParts, GeographicCoordinates, GroundPosition, Member, Members, OsmMap,
};
use crate::osm2layers::Osm2Layer;

///////////////////////////////////////////////////////////////////////////////////////////////////
// openstreetmap_api //////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct InputOsm {
    api_url: String,
    client: Openstreetmap,
}

impl Default for InputOsm {
    fn default() -> Self {
        Self::new()
    }
}

impl InputOsm {
    pub fn new() -> Self {
        let api_url = "https://api.openstreetmap.org/api".into();
        let client = Openstreetmap::new(&api_url, Credentials::None);
        Self { api_url, client }
    }

    pub fn way_url(&self, way_id: u64) -> String {
        format!("{}way/{}/full.json", self.api_url, way_id)
    }

    pub fn bbox_url(&self, bounding_box: &BoundingBox) -> String {
        // https://wiki.openstreetmap.org/wiki/API_v0.6#Retrieving_map_data_by_bounding_box:_GET_/api/0.6/map
        // GET   /api/0.6/map?bbox=left,bottom,right,top
        format!("{}map.json?bbox={}", self.api_url, bounding_box)
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
    ) -> Result<BuildingsAndParts, OpenstreetmapError> {
        let bounding_box = openstreetmap_api::types::BoundingBox {
            left: bounding_box.west as f64,
            bottom: bounding_box.south as f64,
            right: bounding_box.east as f64,
            top: bounding_box.north as f64,
        };
        let map = self.client.map(&bounding_box).await?;

        let mut osm2layer = Osm2Layer::create(*gpu_ground_null_coordinates, show_only);

        for node_ in map.nodes {
            osm2layer.add_node(node_.id, node_.lat.unwrap(), node_.lon.unwrap(), None);
        }

        for way_ in map.ways {
            let mut nodes: Vec<u64> = Vec::new();
            for node in way_.node_refs {
                nodes.push(node.node_id);
            }

            let mut tags: OsmMap = HashMap::new();
            for tag in &way_.tags {
                tags.insert(tag.k.clone(), tag.v.clone());
            }

            osm2layer.add_way(way_.id, nodes, Some(tags));
        }

        for relation_ in map.relations {
            let mut tags: OsmMap = HashMap::new();
            for tag in &relation_.tags {
                tags.insert(tag.k.clone(), tag.v.clone());
            }

            let mut members: Members = Vec::new();
            for member in relation_.members {
                let member = Member {
                    relation_type: member.member_type,
                    reference: member.node_id,
                    role: member.role,
                };
                members.push(member);
            }

            osm2layer.add_relation(relation_.id, members, Some(tags));
        }

        osm2layer.process_elements();

        Ok(osm2layer.get_buildings_and_parts()) //  .buildings_and_parts)
    }
}
