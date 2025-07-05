/////////////////////////////////////////
// Converting OSM data to "GIS" layers //
/////////////////////////////////////////

use csscolorparser::parse;
use std::collections::HashMap;

use crate::footprint::{Footprint, Orientation};
use crate::kernel_in::{
    BuildingOrPart, BuildingsAndParts, GeographicCoordinates, GroundPosition, RenderColor,
    RoofShape,
};
use crate::kernel_in::{Member, Polygons};

// This constands may come from a (3D-)render shema
pub static DEFAULT_WALL_COLOR: RenderColor = [0.5, 0.5, 0.5, 1.0]; // "grey" = RenderColor = [0.5, 0.5, 0.5, 1.0];
pub static DEFAULT_ROOF_COLOR: RenderColor = [1.0, 0.0, 0.0, 1.0]; //  "red"  = RenderColor = [1.0, 0.0, 0.0, 1.0];
pub static DEFAULT_WALL_HEIGHT: f32 = 6.0; // two floors with each 3 meters
pub static DEFAULT_ROOF_HEIGHT: f32 = 2.0;
pub static DEFAULT_MIN_HEIGHT: f32 = 2.0;

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

fn parse_color(color: Option<&String>, default: RenderColor) -> RenderColor {
    // https://docs.rs/csscolorparser/latest/csscolorparser/
    // Bevy pbr color needs f32, The parse has no .as_f32}
    if color.is_none() {
        return default;
    }

    let color_string = color.unwrap().as_str();
    //println!("colour: {} ", color_string);
    match parse(color_string) {
        Ok(color_scc) => {
            //println!("parse_colour: {:?} => {:?}", color, color_scc); // is f64: color_scc.to_array();
            [
                color_scc.r as f32,
                color_scc.g as f32,
                color_scc.b as f32,
                color_scc.a as f32,
            ]
        }

        //        let x = Color::to_array(&self)
        Err(_error) => {
            match color_string {
                "stone" => color_to_f32(200, 200, 200),
                "brick" => color_to_f32(255, 128, 128),
                "cream" => color_to_f32(255, 253, 208),
                "roof_tiles" => color_to_f32(186, 86, 37),
                "glass" => color_to_f32(150, 150, 220), // #light grey wiht a bit blue
                "wood" => color_to_f32(145, 106, 47),
                "copper" => color_to_f32(98, 190, 119), // Verdigris (Grünspahn) instead of copper = 183 119 41

                _ => {
                    println!("parse_colour: {} => {}", color_string, _error);
                    [98. / 255., 203. / 255., 232. / 255., 1.] // Electric Blue (or default???)
                }
            }
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

fn tags_get_yes<'a>(tags: &'a HashMap<String, String>, searched: &str) -> Option<&'a String> {
    if let Some(tag) = tags.get(searched) {
        if tag == "no" { None } else { Some(tag) }
    } else {
        None
    }
}

fn tags_get2<'a>(
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

fn tags_get3<'a>(
    tags: &'a HashMap<String, String>,
    option1: &str,
    option2: &str,
    option3: &str,
) -> Option<&'a String> {
    if let Some(tag) = tags.get(option1) {
        Some(tag)
    } else {
        if let Some(tag) = tags.get(option2) {
            Some(tag)
        } else {
            tags.get(option3)
        }
    }
}

//////////////////////////////// Osm2Layer (API) //////////////////////////////

pub struct OsmNode {
    pub position: GroundPosition,
}

#[derive(Debug, Clone)]
pub struct OsmWay {
    pub _id: u64,
    pub footprint: Footprint,
    pub tags: Option<HashMap<String, String>>,
}

#[derive(Debug)]
pub struct OsmRelation {
    pub id: u64,
    pub members: Vec<Member>,
    pub tags: Option<HashMap<String, String>>,
}

pub struct Osm2Layer {
    gpu_ground_null_coordinates: GeographicCoordinates,
    nodes_map: HashMap<u64, OsmNode>,
    ways_map: HashMap<u64, OsmWay>,
    buildings: Vec<u64>,
    parts: Vec<u64>,
    relations: Vec<OsmRelation>,
    buildings_or_parts: BuildingsAndParts,
    show_only: u64,
}

impl Osm2Layer {
    pub fn create(gpu_ground_null_coordinates: GeographicCoordinates, show_only: u64) -> Self {
        Self {
            gpu_ground_null_coordinates,
            nodes_map: HashMap::new(),
            ways_map: HashMap::new(),
            buildings: Vec::new(),
            parts: Vec::new(),
            relations: Vec::new(),
            buildings_or_parts: Vec::new(),
            show_only,
        }
    }

    pub fn get_buildings_and_parts(self) -> BuildingsAndParts {
        self.buildings_or_parts
    }

    ///////////////////////

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

    ///////////////////////

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

        let mut footprint = Footprint::new(id);
        for node_id in nodes {
            let position = self.nodes_map.get(&node_id).unwrap().position;
            footprint.push_position(position);
        }
        footprint.close();

        if let Some(tags) = &tags {
            if tags_get_yes(&tags, "building").is_some() {
                self.buildings.push(id);
            }
            if tags_get_yes(&tags, "building:part").is_some() {
                self.parts.push(id);
            }
        }

        // Now, as the tags are checked, they may get moved into the map
        self.ways_map.insert(
            id,
            OsmWay {
                _id: id,
                footprint,
                tags,
            },
        );
    }

    ///////////////////////

    pub fn add_relation(
        &mut self,
        id: u64,
        members: Vec<Member>,
        tags: Option<HashMap<String, String>>,
    ) {
        if tags.is_none() {
            println!("Relation without tags: {id}");
            return;
        }

        let tags = tags.unwrap();
        if tags_get_yes(&tags, "building:part").is_none() {
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

    pub fn process_elements_from_osm_to_layers(&mut self) {
        // Subtract parts from ways - code is to stupide! Todo!
        // Test1: cargo run --example m_async -- -r 0 -w 239592652
        // This way does NOT set the height/levels to maximum
        // Test2: building: 278033600 + two parts: 1124726437 1124499584
        // Test3: 278033615 1125067806 todo: part is > building! Subtraktion deletes level 0
        // Test4: rel 2111466 Outer=building 157880278 Parts 109458125 1104998081 1104998082
        // Test5: way 111354081 parts: 814784273 + 814784274 + 814784275
        // Test6: rel 11192041: outer 172664019, inner 814784298
        // Test7: building: 440100162 - parts: 107643530, 440100141 = empty  Todo: tunnel=building_passage

        println!("\n**** process: {:?} relations", self.relations.len());
        while !self.relations.is_empty() {
            //for osm_relation in self.relations.iter() {
            let osm_relation = self.relations.pop().unwrap();
            let way_from_relation = self.process_relation(osm_relation.id, &osm_relation);
            if way_from_relation.is_some() {
                self.ways_map
                    .insert(osm_relation.id, way_from_relation.unwrap());
                self.parts.push(osm_relation.id); // To subtract it from a building, it must be a part! Even it is a building, it works anyway.
            }
        }

        println!("\n**** process {:?} ways", self.buildings.len());
        for building_id in &self.buildings.clone() {
            println!("building: {building_id}");
            let mut building = self.ways_map.remove(building_id).unwrap();

            // substract parts from building
            for part_id in &self.parts {
                //println!("part: {part_id}");
                //if *part_id > 464090146 {
                //    continue;
                //}
                let part = &self.ways_map.get(part_id).unwrap();
                //let shape = part.footprint.clone();
                let part_polygons = part.footprint.polygons.clone(); // todo: avoid clone! how? by ref clashes with ownership
                building.footprint.subtract(&part_polygons);
                if building.footprint.polygons.is_empty() {
                    break;
                }
            }

            if !building.footprint.polygons.is_empty() {
                self.create_building_or_part(*building_id, &mut building);
            }
        }

        println!("\n**** process {:?} parts", self.parts.len());
        for part_id in &self.parts.clone() {
            println!("part: {part_id}");
            let mut part = self.ways_map.remove(part_id).unwrap();
            self.create_building_or_part(*part_id, &mut part);
        }
    }

    ///////////////////////

    // Souldn't we have MORE sub fn's ???
    fn create_building_or_part(&mut self, id: u64, osm_way: &mut OsmWay) {
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
            tags_get3(tags, "building:colour", "colour", "building:material"),
            DEFAULT_WALL_COLOR,
        );
        // Should parts for default get the red DEFAULT_ROOF_COLOR or DEFAULT_WALL_COLOR or the given wall color?
        let roof_color = parse_color(
            tags_get2(tags, "roof:colour", "roof:material"), // todo: parse_material
            if part {
                building_color
            } else {
                DEFAULT_ROOF_COLOR
            },
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

        // todo: parse_direction
        let mut roof_angle = osm_way.footprint.longest_angle;
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
        } else {
            // ... the default along needs a rotation.
            // ttt roof_angle = circle_limit(roof_angle + f32::to_radians(90.));
        }

        let roof_direction = /*parse_orientation???*/ tags.get("roof:direction");
        if let Some(direction) = roof_direction {
            //println!("roof:direction {direction}");
            orienaton_by = Orientation::ByNauticDirction;
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
                        roof_angle = circle_limit(f32::to_radians(value));
                        orienaton_by = Orientation::ByAngleValue;
                    } else {
                        println!("Uncoded roof direction value: {}", direction);
                    }
                }
            }
        }

        // This crate interprets, opposite to OSM the angle along the roof ceiling. Change this???
        roof_angle = circle_limit(roof_angle - f32::to_radians(90.));

        // Not here, in the fn rotate against the actual angle to got 0 degrees
        let (bounding_box_rotated, is_across) = osm_way.footprint.rotate(roof_angle);

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

        if check_across {
            println!(
                "is_across: {is_across} set: {set_across} check: {check_across} roof_angle: {roof_angle}"
            );
            if is_across != set_across {
                //    roof_angle = circle_limit(roof_angle - f32::to_radians(90.));
                //    (_, _) = osm_way.footprint.rotate(roof_angle);
            }
        }

        let building_or_part = BuildingOrPart {
            id,
            part,
            footprint: osm_way.footprint.clone(),
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
        };

        self.buildings_or_parts.push(building_or_part);
    }

    ///////////////////////

    fn process_relation(&mut self, id: u64, osm_relation: &OsmRelation) -> Option<OsmWay> {
        if self.show_only > 0 && id != self.show_only {
            return None;
        }

        // todo: process relation type building? The outer and parts are processed by the normal code anyway, are they?
        // except the outer has no tags!

        /******* self.relation(*id, osm_relation); //  ****/

        println!("Relation: {:?}", id);

        if osm_relation.members.is_empty() {
            println!("Relation without members! id: {:?}", id);
            return None;
        }

        let members = osm_relation.members.clone();

        let tags = osm_relation.tags.as_ref().unwrap();
        // thread 'main' panicked at src/osm2layers.rs:376:42:    cargo run --example m_async -- -r 1000   member with type ""
        let mut relation_type_option = tags.get("type");
        let multipolygon = "multipolygon".to_string();
        if relation_type_option.is_none() {
            println!("relation {id} has no type (and a member without type?).");
            // asume multipolygon (without inner)  todo: code is merde!
            relation_type_option = Some(&multipolygon);
        }
        let relation_type = relation_type_option.unwrap();
        if relation_type != "multipolygon" {
            //println!("Unprocessed relation type: {relation_type}");
            return None;
        }

        //println!("rel tags: {:?}", tags);
        let part_option = tags_get2(tags, "building:part", "building");
        if part_option.is_none() && self.show_only == 0 {
            //println!("Unprocessed relation non-part tag {}", element.id);
            return None;
        }

        let mut footprint = Footprint::new(id);
        let mut outer_id: u64 = 0;

        // first scann for outer, later vo inner
        for member in &members {
            // println!("mem: {:?}", &member);
            if member.relation_type != "way" {
                return None;
            }
            match member.role.as_str() {
                "outer" => {
                    let outer_ref = member.reference;
                    let option = self.ways_map.get(&outer_ref);
                    if option.is_none() {
                        println!("outer none, id/ref: {}", outer_ref);
                        return None;
                    }
                    println!("outer: {}", outer_ref);
                    // Todo: cloning footprint twice can't be the solution
                    let way_with_outer = self.ways_map.get(&outer_ref).unwrap();
                    outer_id = way_with_outer._id;
                    footprint = way_with_outer.footprint.clone();
                }
                _ => (),
            }
        }

        for member in &members {
            //println!("mem: {:?}", &member);
            match member.role.as_str() {
                "inner" => {
                    self.process_relation_inner(member.reference, &mut footprint);
                }
                _ => (),
            }
        }

        if footprint.polygons.is_empty() {
            println!("relation 1");
            return None;
        }

        // buildings_and_parts.push...
        Some(OsmWay {
            _id: outer_id,
            footprint,
            tags: Some(tags.clone()),
        })
    }

    ///////////////////////

    fn process_relation_inner(&self, elements_ref: u64, footprint: &mut Footprint) {
        //println!("elements_ref: {:?}", &elements_ref);
        let option = self.ways_map.get(&elements_ref);
        if option.is_none() {
            println!("inner none, id/ref: {}", elements_ref);
            return;
        }
        println!("inner: {}", elements_ref);

        // todo: what if the hole is has holes? What if the polygon is a multipolygon?
        let polygons: &Polygons = &self.ways_map.get(&elements_ref).unwrap().footprint.polygons;
        footprint.subtract(&polygons);
        //println!("inner way; {:?}", &elements_ref);
    }
}
