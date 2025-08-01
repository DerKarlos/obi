/////////////////////////////////////////
// Converting OSM data to "GIS" layers //
/////////////////////////////////////////

//use bevy::prelude::info;
use csscolorparser::parse;
use std::collections::HashMap;

use crate::footprint::{Footprint, Orientation};
use crate::kernel_in::Members;
use crate::kernel_in::{
    BuildingOrPart, BuildingsAndParts, FIRST_POLYGON, GeographicCoordinates, GroundPosition,
    GroundPositions, OUTER_POLYGON, OsmMap, RenderColor, RoofShape,
};

// This constands may come from a (3D-)render shema
pub static DEFAULT_WALL_COLOR: RenderColor = [0.7, 0.7, 0.7, 1.0]; // "grey" = RenderColor = [0.5, 0.5, 0.5, 1.0];
pub static DEFAULT_ROOF_RED: RenderColor = [0.56, 0.0, 0.0, 1.0]; //  "red"  = RenderColor = [1.0, 0.0, 0.0, 1.0];
pub static DEFAULT_WALL_HEIGHT: f32 = 6.0; // two floors with each 3 meters
pub static DEFAULT_ROOF_HEIGHT: f32 = 2.0;
pub static DEFAULT_MIN_HEIGHT: f32 = 2.0;
pub static DEFAULT_BAD_COLOR: [f32; 4] = [98. / 255., 203. / 255., 232. / 255., 1.]; // Electric Blue

#[derive(PartialEq)]
enum OuterState {
    New,
    Partly,
    Ready,
}

// Helper functions for the osm to layer processing ///////////////////////////

fn circle_limit(angle: f32) -> f32 {
    if angle > f32::to_radians(180.) {
        angle - f32::to_radians(360.)
    } else if angle < f32::to_radians(180.) {
        angle + f32::to_radians(360.)
    } else {
        angle
    }
}

fn parse_color(color: Option<&String>, default: RenderColor, id: u64) -> RenderColor {
    // https://docs.rs/csscolorparser/latest/csscolorparser/
    // Bevy pbr color needs f32, The parse has no .as_f32}
    if color.is_none() {
        return default;
    }

    let mut color_string = color.unwrap().as_str();
    //println!("colour: {} ", color_string);

    const LIGHT: &str = "light";
    const DARK: &str = "dark";
    let mut enlight: f32 = 1.0;
    if color_string.starts_with(LIGHT) {
        color_string = color_string.get(LIGHT.len()..).unwrap();
        enlight = 1.3;
    }
    if color_string.starts_with(DARK) {
        color_string = color_string.get(DARK.len()..).unwrap();
        enlight = 1. / 1.3;
    }
    if color_string.starts_with('_') {
        color_string = color_string.get(1..).unwrap();
    }

    let color_or_error = parse(color_string); // <<<======= parse color =========

    if color_or_error.is_ok() {
        let color_scc = color_or_error.unwrap();
        return [
            color_scc.r as f32 * enlight,
            color_scc.g as f32 * enlight,
            color_scc.b as f32 * enlight,
            color_scc.a as f32 * enlight,
        ];
    }

    match color_string {
        // yellow-brown
        "metal" => color_to_f32(70, 71, 62),
        "sandstone" => color_to_f32(191, 166, 116),
        "slate" => color_to_f32(112, 128, 144),
        "concrete" => color_to_f32(196, 182, 166),
        "stone" => color_to_f32(200, 200, 200),
        "brick" => color_to_f32(255, 128, 128),
        "cream" => color_to_f32(255, 253, 208),
        "roof_tiles" => color_to_f32(186, 86, 37),
        "glass" => color_to_f32(150, 150, 220), // #light grey wiht a bit blue
        "wood" => color_to_f32(145, 106, 47),
        "copper" => color_to_f32(98, 190, 119), // Verdigris (Grünspahn) instead of copper = 183 119 41

        _ => {
            println!(
                "OSM id {id}: Bad color: {}",
                color_string,
                //color_or_error.err()
            );
            DEFAULT_BAD_COLOR
        }
    }
}

fn color_to_f32(r: u8, g: u8, b: u8) -> RenderColor {
    [r as f32 / 255., g as f32 / 255., b as f32 / 255., 1.]
}

fn parse_height(height_option: Option<&String>) -> f32 {
    if height_option.is_none() {
        return 0.;
    }

    let mut height = height_option.unwrap().clone();

    if height.ends_with("m") {
        height = height.strip_suffix("m").unwrap().into();
    }

    match height.as_str().trim().parse() {
        Ok(height) => height,

        Err(error) => {
            println!("Error! parse_height: {} for:{}:", error, height);
            0.
        }
    }
}

fn tags_get_yes<'a>(tags: &'a OsmMap, searched: &str) -> Option<&'a String> {
    if let Some(tag) = tags.get(searched) {
        if tag == "no" { None } else { Some(tag) }
    } else {
        None
    }
}

fn tags_get2<'a>(tags: &'a OsmMap, option1: &str, option2: &str) -> Option<&'a String> {
    if let Some(tag) = tags.get(option1) {
        Some(tag)
    } else {
        tags.get(option2)
    }
}

fn tags_get3<'a>(
    tags: &'a OsmMap,
    option1: &str,
    option2: &str,
    option3: &str,
) -> Option<&'a String> {
    if let Some(tag) = tags.get(option1) {
        Some(tag)
    } else if let Some(tag) = tags.get(option2) {
        Some(tag)
    } else {
        tags.get(option3)
    }
}

#[derive(Debug, Clone)]

struct OsmLine {
    id: u64,
    positions: GroundPositions,
    tags: Option<OsmMap>,
}

impl Default for OsmLine {
    fn default() -> Self {
        Self::new(4714)
    }
}

impl OsmLine {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            positions: Vec::new(),
            tags: None,
        }
    }
}

//////////////////////////////// Osm2Layer (API) //////////////////////////////

pub struct OsmNode {
    pub position: GroundPosition,
}

#[derive(Debug, Clone)]
pub struct OsmArea {
    pub _id: u64,
    pub footprint: Footprint,
    pub tags: Option<OsmMap>,
}

#[derive(Debug)]
pub struct OsmRelation {
    pub id: u64,
    pub members: Members,
    pub tags: Option<OsmMap>,
}

pub struct Osm2Layer {
    gpu_ground_null_coordinates: GeographicCoordinates,
    nodes_map: HashMap<u64, OsmNode>,
    areas_map: HashMap<u64, OsmArea>,
    lines_map: HashMap<u64, OsmLine>,
    buildings: Vec<u64>,
    parts: Vec<u64>,
    relations: Vec<OsmRelation>,
    outer_state: OuterState,
    buildings_or_parts: BuildingsAndParts,
    show_only: u64,
}

impl Osm2Layer {
    pub fn create(gpu_ground_null_coordinates: GeographicCoordinates, show_only: u64) -> Self {
        Self {
            gpu_ground_null_coordinates,
            nodes_map: HashMap::new(),
            areas_map: HashMap::new(),
            lines_map: HashMap::new(),
            buildings: Vec::new(),
            parts: Vec::new(),
            outer_state: OuterState::New,
            relations: Vec::new(),
            buildings_or_parts: Vec::new(),
            show_only,
        }
    }

    pub fn get_buildings_and_parts(self) -> BuildingsAndParts {
        self.buildings_or_parts
    }

    ///////////////////////

    pub fn add_node(&mut self, id: u64, latitude: f64, longitude: f64, _tags: Option<OsmMap>) {
        self.nodes_map.insert(
            id,
            OsmNode {
                position: self
                    .gpu_ground_null_coordinates
                    .coordinates_to_position(latitude, longitude),
            },
        );
    }

    pub fn add_way(&mut self, id: u64, mut nodes: Vec<u64>, tags: Option<OsmMap>) {
        // Only closed ways (yet)
        if nodes.first().unwrap() == nodes.last().unwrap() {
            if nodes.len() < 3 {
                println!("Closed way with < 3 corners! id: {}", id);
                return;
            }
            nodes.pop();
            self.add_area(id, nodes, tags);
        } else {
            self.add_line(id, nodes, tags);
        }
    }

    pub fn add_line(&mut self, id: u64, nodes: Vec<u64>, tags: Option<OsmMap>) {
        let mut positions = Vec::new();
        for node_id in nodes {
            let position = self.nodes_map.get(&node_id).unwrap().position;
            positions.push(position);
        }
        //??? line.close();
        self.lines_map.insert(
            id,
            OsmLine {
                id,
                positions,
                tags,
            },
        );
    }

    pub fn add_area(&mut self, id: u64, nodes: Vec<u64>, tags: Option<OsmMap>) {
        let mut footprint = Footprint::new();
        for node_id in nodes {
            let position = self.nodes_map.get(&node_id).unwrap().position;
            footprint.push_position(position);
        }
        footprint.close();

        // When needs a buidling als to be a part? This example is just a building:
        // https://www.openstreetmap.org/edit#map=22/51.4995203/-0.1290937
        // So building else if solves it??? Overpass vor beeng both and check
        if let Some(tags) = &tags {
            if tags_get_yes(tags, "building").is_some() {
                self.buildings.push(id);
            } else if tags_get_yes(tags, "building:part").is_some() {
                self.parts.push(id);
            }
        }

        // Now, as the tags are checked, they may get moved into the map
        self.areas_map.insert(
            id,
            OsmArea {
                _id: id,
                footprint,
                tags,
            },
        );
    }

    pub fn add_relation(&mut self, id: u64, members: Members, tags: Option<OsmMap>) {
        if tags.is_none() {
            println!("Relation without tags: {id}");
            return;
        }

        let tags = tags.unwrap();
        if tags_get_yes(&tags, "building:part").is_none()
            && tags_get_yes(&tags, "building").is_none()
        {
            // if show_only = 0
            return;
        }

        self.relations.push(OsmRelation {
            id,
            members,
            tags: Some(tags),
        });
    }

    ///////////////////////

    pub fn process_elements(&mut self) {
        //
        #[cfg(debug_assertions)]
        println!("\n**** process: {:?} relations", self.relations.len());
        while let Some(mut osm_relation) = self.relations.pop() {
            self.process_relation(osm_relation.id, &mut osm_relation);
        }

        #[cfg(debug_assertions)]
        println!("\n**** process {:?} ways", self.buildings.len());
        // Bevy function does not work here info!("\n**** process {:?} ways", self.buildings.len());
        while let Some(building_id) = self.buildings.pop() {
            // while !self.buildings.is_empty() {
            //let building_id = self.buildings.pop().unwrap();
            let mut building = self.areas_map.remove(&building_id).unwrap();
            let initial_area_size = building.footprint.get_area_size();

            // Subtract parts from building outer ways - code is slow? Todo!
            for part_id in &self.parts {
                //println!("part: {part_id}");
                //if *part_id > 814784299 {
                //    continue;
                //}
                let part = &self.areas_map.get(part_id).unwrap();
                building.footprint.subtract(&part.footprint);
                if building.footprint.polygons.is_empty() {
                    break;
                }
            }

            let remaining_area_size = building.footprint.get_area_size();
            let percent_left = (remaining_area_size / initial_area_size * 100.) as i32;

            #[cfg(debug_assertions)]
            println!("building: {building_id} left: {percent_left}%");

            if !building.footprint.polygons.is_empty() && percent_left >= 40 {
                self.create_building_or_part(building_id, &mut building);
            }
        }

        #[cfg(debug_assertions)]
        println!("\n**** process {:?} parts", self.parts.len());
        for part_id in &self.parts.clone() {
            #[cfg(debug_assertions)]
            println!("part: {part_id}");
            let mut part = self.areas_map.remove(part_id).unwrap();
            self.create_building_or_part(*part_id, &mut part);
        }
    }

    ///////////////////////

    // Souldn't we have MORE sub fn's ???
    fn create_building_or_part(&mut self, id: u64, osm_way: &mut OsmArea) {
        //println!("scan: way id = {:?}", id);
        if self.show_only > 0 && id != self.show_only {
            return;
        }

        let tags = osm_way.tags.as_ref().unwrap();

        if osm_way.footprint.polygons.is_empty() {
            println!("create_building_or_part: way is empty {:?}", id);
            return;
        }

        // // // // // // // // //

        let part = tags.get("building:part").is_some();
        let simple_footprint = osm_way.footprint.polygons.len() == 1
            && osm_way.footprint.polygons[FIRST_POLYGON][OUTER_POLYGON].len() <= 6;

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
                    if simple_footprint {
                        RoofShape::Gabled // todo: geographic dependend ggg
                    } else {
                        RoofShape::Flat
                    }
                }
            },
            None => RoofShape::Flat,
        };

        // ** Colors and Materials **
        let building_color = parse_color(
            tags_get3(tags, "building:colour", "colour", "building:material"),
            DEFAULT_WALL_COLOR,
            id,
        );
        // Should parts for default get the red DEFAULT_ROOF_COLOR or DEFAULT_WALL_COLOR or the given wall color?
        let roof_color = parse_color(
            tags_get2(tags, "roof:colour", "roof:material"), // todo: parse_material
            if part {
                building_color
            } else {
                // if it is a simple building with only a view corners: Red roof (and gabled???)
                if simple_footprint {
                    DEFAULT_ROOF_RED
                } else {
                    DEFAULT_WALL_COLOR
                }
            },
            id,
        );

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
        let roof_levels = parse_height(tags.get("roof:levels"));
        if roof_height == 0. && roof_levels > 0. {
            roof_height = roof_levels * 3.0;
        }
        if roof_height == 0. {
            roof_height = default_roof_heigt;
        }
        //println!( "roof_height: {roof_height} default_roof_heigt: {default_roof_heigt} roof_shape: {:?}", roof_shape);
        //let wall_height = parse_height(tags.get("height"), 6.0 /*DEFAULT_WALL_HEIGHT*/) - roof_height;

        let mut building_height = parse_height(tags_get2(tags, "building:height", "height"));
        let levels = parse_height(tags_get2(tags, "building:levels", "building:levels"));
        if building_height == 0. && levels > 0. {
            building_height = levels * 3.0 + roof_height;
        }
        if building_height == 0. {
            building_height = DEFAULT_WALL_HEIGHT;
        }
        let wall_height = building_height - roof_height;

        // ** Roof direction and Orientation **

        // The longest angle sets the dirction of the ceiling. But the tagging value is along the slope!
        let mut roof_angle = circle_limit(osm_way.footprint.longest_angle + f32::to_radians(90.0));
        let roof_orientation = tags.get("roof:orientation");
        let mut orienaton_by: Orientation = Orientation::ByLongestSide;
        // https://wiki.openstreetmap.org/wiki/Key:roof:orientation

        // Note! In OSM, the roof angle is along the roof slope! It is ot along the roof ridge!
        // Wired!: OSM defines the roof-angle value as across the lonest way side! So, ...
        if let Some(orientation) = roof_orientation {
            match orientation.as_str() {
                "along" => orienaton_by = Orientation::Along,
                "across" => orienaton_by = Orientation::Across,
                _ => println!("Uncoded roof orientation value: {}", orientation),
            }
        }

        let roof_direction = /*parse_orientation???*/ tags.get("roof:direction");
        if let Some(direction) = roof_direction {
            //println!("roof:direction {direction}");
            orienaton_by = Orientation::ByNauticDirction;
            match direction.as_str() {
                "N" => roof_angle = f32::to_radians(0.),
                "E" => roof_angle = f32::to_radians(90.),
                "S" => roof_angle = f32::to_radians(180.),
                "W" => roof_angle = f32::to_radians(270.),

                "NE" => roof_angle = f32::to_radians(45.),
                "SE" => roof_angle = f32::to_radians(135.),
                "SW" => roof_angle = f32::to_radians(225.),
                "NW" => roof_angle = f32::to_radians(315.),

                "NNE" => roof_angle = f32::to_radians(22.),
                "ENE" => roof_angle = f32::to_radians(67.),
                "ESE" => roof_angle = f32::to_radians(112.),
                "SSE" => roof_angle = f32::to_radians(157.),

                "SSW" => roof_angle = f32::to_radians(202.),
                "WSW" => roof_angle = f32::to_radians(247.),
                "WNW" => roof_angle = f32::to_radians(292.),
                "NNW" => roof_angle = f32::to_radians(337.),
                _ => {
                    let value = direction.parse();
                    if let Ok(value) = value {
                        roof_angle = circle_limit(f32::to_radians(value));
                        orienaton_by = Orientation::ByAngleValue;
                    } else {
                        println!("Uncoded roof direction value: {}", direction);
                    }
                }
            }
        }

        // Not here at the parameter, but in the fn rotate against the actual angle to got 0 degrees
        let mut bounding_box_rotated = osm_way.footprint.rotate(roof_angle);

        let mut check_across = false;
        let mut set_across = false;
        match orienaton_by {
            // OSM default is across
            Orientation::ByLongestSide => {
                check_across = true;
            }
            Orientation::Along => {
                check_across = true;
            }
            Orientation::Across => {
                check_across = true;
                set_across = true
            }

            _ => (),
        };

        let is_across = bounding_box_rotated.north - bounding_box_rotated.south
            > bounding_box_rotated.east - bounding_box_rotated.west;

        // In Bakerboys longestSideAngle is this across-correction too
        if check_across {
            //println!(    "is_across: {is_across} set: {set_across} check: {check_across} roof_angle: {roof_angle} - bboxr: {:?}", bounding_box_rotated);
            if is_across != set_across {
                roof_angle = circle_limit(roof_angle + f32::to_radians(90.));
                bounding_box_rotated = osm_way.footprint.rotate(roof_angle);
            }
        }

        let building_or_part = BuildingOrPart {
            id,
            part,
            footprint: osm_way.footprint.clone(),
            bounding_box_rotated,
            wall_height,
            min_height,
            building_color,
            roof_shape,
            roof_height,
            roof_angle,
            roof_color,
        };

        self.buildings_or_parts.push(building_or_part);
    }

    ///////////////////////

    fn process_relation(&mut self, id: u64, osm_relation: &mut OsmRelation) {
        if self.show_only > 0 && id != self.show_only {
            return;
        }

        // todo: process relation type building? The outer and parts are processed by the normal code anyway, are they?
        // except the outer has no tags!

        #[cfg(debug_assertions)]
        println!("Relation: {:?}", id);

        if osm_relation.members.is_empty() {
            println!("Relation without members! id: {:?}", id);
            return;
        }

        let members = osm_relation.members.clone();

        let tags = osm_relation.tags.as_ref().unwrap();
        // panicked at src/osm2layers.rs:376:42:    cargo run --example m_async -- -r 1000   member with type ""
        let mut relation_type_option = tags.get("type");
        let multipolygon = "multipolygon".into();
        if relation_type_option.is_none() {
            println!("relation {id} has no type (and a member without type?).");
            // asume multipolygon (without inner)  todo: code is merde!
            relation_type_option = Some(&multipolygon);
        }
        let relation_type = relation_type_option.unwrap();
        if relation_type != "multipolygon" {
            //println!("Unprocessed relation type: {relation_type}");
            return;
        }

        //println!("rel tags: {:?}", tags);
        let part_option = tags_get2(tags, "building:part", "building");
        if part_option.is_none() && self.show_only == 0 {
            //println!("Unprocessed relation non-part tag {}", id);
            return;
        }

        let mut footprint = Footprint::new();

        self.outer_state = OuterState::New;

        // first scann for outer, later vo inner
        for member in &members {
            // println!("mem: {:?}", &member);
            if member.member_type != "way" {
                return;
            }

            if member.role.as_str() == "outer" {
                self.process_relation_outer(member.reference, &mut footprint, id);
            }
        }

        for member in &members {
            //println!("mem: {:?}", &member);
            if member.role.as_str() == "inner" {
                self.process_relation_inner(member.reference, &mut footprint, id);
            }
        }

        if footprint.polygons.is_empty() {
            println!("relation 1");
            return;
        }

        if self.outer_state == OuterState::Partly {
            footprint.close();
        }

        // buildings_and_parts.push...
        let v = OsmArea {
            _id: id,
            footprint,
            tags: Some(tags.clone()),
        };

        self.areas_map.insert(id, v);
        let part = tags_get_yes(osm_relation.tags.as_ref().unwrap(), "building:part").is_some();

        if part {
            self.parts.push(osm_relation.id); // To subtract it from a building, it must be a part
        } else {
            self.buildings.push(osm_relation.id); // If IT is a building, it must be in the building list, to get parts substracted!
        }
    }

    ///////////////////////

    fn process_relation_outer(&mut self, outer_ref: u64, new_footprint: &mut Footprint, id: u64) {
        #[cfg(debug_assertions)]
        println!("outer: {}", outer_ref);
        if let Some(area) = self.areas_map.get(&outer_ref) {
            // Todo: cloning footprint twice can't be the solution
            if self.outer_state != OuterState::New {
                println!("Relation {id}: Odd outer way: {}", outer_ref);
                return;
            }
            new_footprint.set(&area.footprint);
            self.outer_state = OuterState::Ready;
            return;
        }

        if let Some(line) = self.lines_map.get(&outer_ref) {
            self.outer_state = OuterState::Partly;
            println!(
                // todo: multi outer/inner
                "outer line, id: {} nodes: {} taggs: {}",
                line.id,
                line.positions.len(),
                line.tags.is_some()
            );
            for position in &line.positions {
                new_footprint.push_position(*position);
            }
            return;
        }
        println!("outer none, id/ref: {}", outer_ref);
    }

    fn process_relation_inner(&self, elements_ref: u64, new_footprint: &mut Footprint, id: u64) {
        //println!("elements_ref: {:?}", &elements_ref);
        let option = self.areas_map.get(&elements_ref);
        if option.is_none() {
            println!("Relaton {id}: Inner way {} not loaded!", elements_ref);
            return;
        }
        #[cfg(debug_assertions)]
        println!("inner: {}", elements_ref);

        // todo: what if the hole is has holes? What if the polygon is a multipolygon?
        let footprint: &Footprint = &self.areas_map.get(&elements_ref).unwrap().footprint;
        new_footprint.subtract(footprint);
        //println!("inner way; {:?}", &elements_ref);
    }
}
