use std::collections::HashMap;
//use std::fmt::Error;

use openstreetmap_api::Openstreetmap;
use openstreetmap_api::errors::OpenstreetmapError;
use openstreetmap_api::types::{Credentials, Node, Way};

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
// static API_URL: &str = "https://api.openstreetmap.org/api/0.6/";

pub struct InputLib {
    client: Openstreetmap,
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

    /********************
     ********************/
}

/********************/

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
    element: Way,
    building_parts: &mut Vec<BuildingPart>,
    nodes_map: &mut HashMap<u64, OsmNode>,
    show_only_this_id: u64,
) {
    // println!("element = {:?}", element);
    if show_only_this_id > 0 && element.id != show_only_this_id {
        return;
    } // tttt

    if element.tags.len() == 0 {
        // ttt println!( "way without tags! ID: {} Relation(-Outer) or Multipolligon?",element.id);
        return;
    }

    let mut tags: HashMap<String, String> = HashMap::new();
    for tag in &element.tags {
        tags.insert(tag.k.clone(), tag.v.clone());
    }

    let string_no = &NO.to_string();
    let part = tags.get("building:part").unwrap_or(string_no);

    // ??? not only parts!
    if part == YES || show_only_this_id > 0 {
        building(element, tags, building_parts, nodes_map);
    }
}

fn building(
    element: Way,
    tags: HashMap<String, String>,
    building_parts: &mut Vec<BuildingPart>,
    nodes_map: &mut HashMap<u64, OsmNode>,
) {
    // Validate way-nodes
    let mut nodes = element.node_refs;
    if nodes.len() < 3 {
        println!("Building with < 3 corners! id: {}", element.id);
        return;
    }
    if nodes.first().unwrap() != nodes.last().unwrap() {
        println!("Building with < 3 corners! id: {}", element.id);
    } else {
        nodes.pop();
    }

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
    for node_ref in nodes.iter() {
        let node = nodes_map.get(&node_ref.node_id).unwrap();
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

/********************/
