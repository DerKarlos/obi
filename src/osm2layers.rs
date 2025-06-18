///////////////////////////////////////
// The tactics to hanlde OSM tagging //
///////////////////////////////////////

use csscolorparser::parse;
use std::collections::HashMap;

use crate::GeographicCoordinates;
use crate::kernel_in::Member;
use crate::kernel_in::{BuildingPart, OsmNode, OsmRelation, OsmWay, RenderColor, RoofShape};
use crate::shape::Shape;

// This constands may come from a (3D-)render shema
pub static DEFAULT_WALL_COLOR: RenderColor = [0.5, 0.5, 0.5, 1.0]; // "grey" = RenderColor = [0.5, 0.5, 0.5, 1.0];
pub static DEFAULT_ROOF_COLOR: RenderColor = [1.0, 0.0, 0.0, 1.0]; //  "red"  = RenderColor = [1.0, 0.0, 0.0, 1.0];
pub static DEFAULT_WALL_HEIGHT: f32 = 6.0; // two floors with each 3 meters
pub static DEFAULT_ROOF_HEIGHT: f32 = 2.0;
pub static DEFAULT_MIN_HEIGHT: f32 = 2.0;

static NO: &str = "no";

pub fn circle_limit(angle: f32) -> f32 {
    if angle > f32::to_radians(180.) {
        angle - f32::to_radians(360.)
    } else if angle < f32::to_radians(180.) {
        angle + f32::to_radians(360.)
    } else {
        angle
    }
}

// MAy return option if once needed
pub fn parse_color(color: Option<&String>, default: RenderColor) -> RenderColor {
    // https://docs.rs/csscolorparser/latest/csscolorparser/
    // Bevy pbr color needs f32, The parse has no .as_f32}
    if color.is_none() {
        return default;
    }

    match parse(color.unwrap().as_str()) {
        Ok(color_scc) => {
            //println!("parse_colour: {:?} => {:?}", color, color_scc);
            [
                color_scc.r as f32,
                color_scc.g as f32,
                color_scc.b as f32,
                color_scc.a as f32,
            ]
        }

        Err(_error) => {
            // println!("parse_colour: {}", _error);
            default // "light blue?"
        }
    }
}

pub fn parse_height(height_option: Option<&String>) -> f32 {
    if height_option.is_none() {
        return 0.;
    }

    let mut height = height_option.unwrap().clone();

    if height.ends_with("m") {
        height = height.strip_suffix("m").unwrap().to_string();
    }

    match height.as_str().trim().parse() {
        Ok(height) => height,

        Err(error) => {
            println!("Error! parse_height: {} for:{}:", error, height);
            0.
        }
    }
}

pub fn tags_get2<'a>(
    tags: &'a HashMap<String, String>,
    option1: &str,
    option2: &str,
) -> Option<&'a String> {
    if let Some(tag) = tags.get(option1) {
        Some(tag)
    } else {
        tags.get(option2)
    }
}

pub fn building(
    footprint: &mut Shape,
    id: u64,
    tags: &HashMap<String, String>,
    //building_parts: &Vec<BuildingPart>,
) -> BuildingPart {
    // ** Shape of the roof. All buildings have a roof, even if it is not tagged **
    let roof_shape: RoofShape = match tags.get("roof:shape") {
        Some(roof_shape) => match roof_shape.as_str() {
            "flat" => RoofShape::Flat,
            "skillion" => RoofShape::Skillion,
            "gabled" => RoofShape::Gabled,
            "pyramidal" => RoofShape::Phyramidal,
            "dome" => RoofShape::Dome,
            "onion" => RoofShape::Onion,
            _ => {
                // println!("Warning: roof_shape Unknown: {}", roof_shape);
                RoofShape::Flat // todo: gabled and geographic dependend
            }
        },
        None => RoofShape::Flat,
    };

    // ** Colors and Materials **
    let building_color = parse_color(
        tags_get2(tags, "building:colour", "colour"),
        DEFAULT_WALL_COLOR,
    );
    let roof_color = parse_color(tags.get("roof:colour"), DEFAULT_ROOF_COLOR);

    println!("fn building: Part id: {} roof: {:?}", id, roof_shape);

    let default_roof_heigt = match roof_shape {
        RoofShape::Flat => 0.0,
        RoofShape::Skillion => 2.0, // todo: accroding to width
        RoofShape::Gabled => 2.0,
        RoofShape::None => 0.0,
        _ => 2.0, //DEFAULT_ROOF_HEIGHT,
    };

    // ** Heights **  // todo: a new fn process_heights
    let min_height = parse_height(tags.get("min_height")); // DEFAULT_MIN_HEIGHT
    let mut roof_height = parse_height(tags.get("roof:height"));
    if roof_height == 0. {
        roof_height = default_roof_heigt;
    }
    //println!( "roof_height: {roof_height} default_roof_heigt: {default_roof_heigt} roof_shape: {:?}", roof_shape);
    //let wall_height = parse_height(tags.get("height"), 6.0 /*DEFAULT_WALL_HEIGHT*/) - roof_height;

    let mut building_height = parse_height(tags_get2(tags, "building:height", "height"));
    let levels = parse_height(tags_get2(tags, "building:levels", "building:levels"));
    if building_height == 0. && levels > 0. {
        building_height = levels * 3.0;
    }
    if building_height == 0. {
        building_height = DEFAULT_WALL_HEIGHT;
    }
    let wall_height = building_height - roof_height;

    // ** Roof direction and Orientation **

    // todo: parse_direction
    let mut roof_angle = footprint.longest_angle;
    let roof_orientation = tags.get("roof:orientation");
    // https://wiki.openstreetmap.org/wiki/Key:roof:orientation

    // Wired!: OSM defines the roof-angle value as across the lonest way side! So, ...
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

    //println!("- roof_angle: {}", roof_angle.to_degrees());

    // This crate interprets, opposite to OSM the angle along the roof ceiling. Change this???
    roof_angle = circle_limit(roof_angle - f32::to_radians(90.));

    // Not here, in the fn rotate against the actual angle to got 0 degrees
    let bounding_box_rotated = footprint.rotate(roof_angle);

    // let building_part =
    BuildingPart {
        _id: id,
        _part: true, // ??? not only parts!
        footprint: footprint.clone(),
        //center,
        //bounding_box: bounding_box,
        bounding_box_rotated,
        wall_height,
        min_height,
        building_color,
        roof_shape,
        roof_height,
        roof_angle,
        roof_color,
    }

    // println!("building_part: {:?}", building_part);
    // building_part
}

//////////////////////////////// Osm2Layer //////////////////////////////

pub struct Osm2Layer {
    gpu_ground_null_coordinates: GeographicCoordinates,
    nodes_map: HashMap<u64, OsmNode>,
    ways_map: HashMap<u64, OsmWay>,
    relations_map: HashMap<u64, OsmRelation>,
    pub building_parts: Vec<BuildingPart>,
    show_only: u64,
}

impl Osm2Layer {
    pub fn create(gpu_ground_null_coordinates: GeographicCoordinates, show_only: u64) -> Self {
        Self {
            gpu_ground_null_coordinates,
            nodes_map: HashMap::new(),
            ways_map: HashMap::new(),
            relations_map: HashMap::new(),
            building_parts: Vec::new(),
            show_only,
        }
    }
    pub fn add_node(
        &mut self,
        id: u64,
        latitude: f64,
        longitude: f64,
        _tags: Option<HashMap<String, String>>,
    ) {
        self.nodes_map.insert(
            id,
            OsmNode {
                position: self
                    .gpu_ground_null_coordinates
                    .coordinates_to_position(latitude, longitude),
            },
        );
    }

    pub fn add_way(&mut self, id: u64, mut nodes: Vec<u64>, tags: Option<HashMap<String, String>>) {
        // Only closed ways (yet)
        if nodes.first().unwrap() != nodes.last().unwrap() {
            //println!("Not a closed way id: {}", element.id);
            return;
        }
        if nodes.len() < 3 {
            println!("Closed way with < 3 corners! id: {}", id);
            return;
        } else {
            nodes.pop();
        }

        let mut footprint = Shape::new();
        for node_id in nodes {
            let position = self.nodes_map.get(&node_id).unwrap().position;
            footprint.push(position);
        }
        footprint.close();
        // println!("add_way insert id: {} ", id);
        self.ways_map.insert(id, OsmWay { footprint, tags });
    }

    pub fn add_relation(
        &mut self,
        id: u64,
        members: Vec<Member>,
        tags: Option<HashMap<String, String>>,
    ) {
        self.relations_map.insert(id, OsmRelation { members, tags });
    }

    pub fn scan(&mut self) {
        println!("scan: way len = {:?}", self.ways_map.len());
        for (id, osm_way) in self.ways_map.iter_mut() {
            way(*id, osm_way, self.show_only, &mut self.building_parts);
        }

        println!("scan: rel len = {:?}", self.relations_map.len());
        for (id, osm_relation) in self.relations_map.iter() {
            relation(
                *id,
                osm_relation,
                &self.ways_map,
                self.show_only,
                &mut self.building_parts,
            );
        }
    }
}

// todo? Is it possible to make this fn as then-fn of Osm2Layer
fn way(id: u64, osm_way: &mut OsmWay, show_only: u64, building_parts: &mut Vec<BuildingPart>) {
    // todo: Fight Rust and make this {} a fn

    //println!("scan: way id = {:?}", id);
    if show_only > 0 && id != show_only {
        return;
    }

    if osm_way.tags.is_none() {
        return;
    }

    let string_no = &NO.to_string();
    let tags = osm_way.tags.as_ref().unwrap();
    let part = tags.get("building:part").unwrap_or(string_no);

    // ??? not only parts!    || show_only < 0
    if part != NO || show_only > 0 {
        building_parts.push(building(&mut osm_way.footprint, id, tags));
    }
}
fn relation(
    id: u64,
    osm_relation: &OsmRelation,
    ways_map: &HashMap<u64, OsmWay>,
    show_only: u64,
    building_parts: &mut Vec<BuildingPart>,
) {
    // println!("scan: rel. id = {:?}", id);
    if show_only > 0 && id != show_only {
        return;
    }

    if osm_relation.tags.is_none() {
        return;
    }

    let string_no = &NO.to_string();
    let tags = osm_relation.tags.as_ref().unwrap();
    let part = tags.get("building:part").unwrap_or(string_no);
    if part == NO && show_only > 0 {
        return;
    }

    /******* self.relation(*id, osm_relation); //  ****/

    // println!("Relation, id: {:?}", id);

    if osm_relation.members.is_empty() {
        println!("Relation without members! id: {:?}", id);
        return;
    }

    let members = osm_relation.members.clone();

    let tags = &osm_relation.tags.as_ref().unwrap();
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
        if member.relation_type != "way" {
            return;
        }
        match member.role.as_str() {
            "outer" => {
                let outer_ref = member.reference;
                let option = ways_map.get(&outer_ref);
                if option.is_none() {
                    println!("outer none, id/ref: {}", outer_ref);
                    return;
                }
                // Todo: cloning footprint twice can't be the solution
                footprint = ways_map.get(&outer_ref).unwrap().footprint.clone();
            }
            "inner" => {
                inner(member.reference, ways_map, &mut footprint);
            }
            _ => (),
        }
    }

    // self.
    building_parts.push(building(&mut footprint, id, tags));
}

fn inner(elements_ref: u64, ways_map: &HashMap<u64, OsmWay>, footprint: &mut Shape) {
    //println!("elements_ref: {:?}", &elements_ref);
    let option = ways_map.get(&elements_ref);
    if option.is_none() {
        println!("inner none, id/ref: {}", elements_ref);
        return;
    }
    let hole = ways_map.get(&elements_ref).unwrap().footprint.clone();
    footprint.push_hole(hole);
    //println!("outer_way; {:?}", &outer_way);
}
