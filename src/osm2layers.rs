///////////////////////////////////////
// The tactics to hanlde OSM tagging //
///////////////////////////////////////

use csscolorparser::parse;
use std::collections::HashMap;

use crate::GeographicCoordinates;
use crate::footprint::Footprint;
use crate::kernel_in::Member;
use crate::kernel_in::{BuildingPart, OtbNode, OtbRelation, OtbWay, RenderColor, RoofShape};

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

pub fn tags_get3<'a>(
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

pub fn building(
    footprint: &mut Footprint,
    id: u64,
    tags: &HashMap<String, String>,
    //building_parts: &Vec<BuildingPart>,
) -> BuildingPart {
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
    // Should parts have the red DEFAULT_ROOF_COLOR or DEFAULT_WALL_COLOR or the given wall color?
    let roof_color = parse_color(
        tags_get2(tags, "roof:colour", "roof:material"), // todo: parse_material
        building_color,
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
        //println!("roof:direction {direction}");
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
                } else {
                    println!("Uncoded roof direction value: {}", direction);
                }
            }
        }
    }

    // println!(
    //     "fn building: Part id: {} roof: {:?} cirular: {} angle: {}",
    //     id,
    //     roof_shape,
    //     footprint.is_circular,
    //     roof_angle.to_degrees()
    // );

    // This crate interprets, opposite to OSM the angle along the roof ceiling. Change this???
    roof_angle = circle_limit(roof_angle - f32::to_radians(90.));

    // Not here, in the fn rotate against the actual angle to got 0 degrees
    let bounding_box_rotated = footprint.rotate(roof_angle);

    // let building_part =
    BuildingPart {
        id,
        part,
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
    nodes_map: HashMap<u64, OtbNode>,
    ways_map: HashMap<u64, OtbWay>,
    buildings: Vec<u64>,
    parts: Vec<u64>,
    relations: Vec<OtbRelation>,
    pub building_parts: Vec<BuildingPart>,
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
            OtbNode {
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

        let mut footprint = Footprint::new(id);
        for node_id in nodes {
            let position = self.nodes_map.get(&node_id).unwrap().position;
            footprint.push_position(position);
        }
        footprint.close();
        // println!(
        //     "add_way insert id: {} circular: {} ",
        //     id, footprint.is_circular
        // );

        if tags.as_ref().is_some() {
            if tags.as_ref().unwrap().get("building").is_some() {
                self.buildings.push(id);
            }

            if tags.as_ref().unwrap().get("building:part").is_some() {
                self.parts.push(id);
            }
        }

        self.ways_map.insert(
            id,
            OtbWay {
                _id: id,
                footprint,
                tags,
            },
        );
    }

    pub fn add_relation(
        &mut self,
        id: u64,
        members: Vec<Member>,
        tags: Option<HashMap<String, String>>,
    ) {
        self.relations.push(OtbRelation { id, members, tags });
    }

    pub fn scan(&mut self) {
        // Subtract parts from ways - code is to stupide! Todo!
        // Test1: cargo run --example m_async -- -r 0 -w 239592652
        // This way does NOT set the height/levels to maximum
        // Test2: building: 278033600 + two parts: 1124726437 1124499584
        // Test3: 278033615 1125067806 todo: part is > building! Subtraktion deletes level 0
        // Test4: rel 2111466 Outer=building 157880278 Parts 109458125 1104998081 1104998082
        // Test5: way 111354081 parts: 814784273 + 814784274 + 814784275
        // Test6: rel 11192041: outer 172664019, inner 814784298
        // Test7: building: 440100162 - parts: 107643530, 440100141 = empty  Todo: tunnel=building_passage

        println!("scan: rel len = {:?}", self.relations.len());
        for osm_relation in self.relations.iter() {
            let way_from_relation = relation(
                osm_relation.id,
                osm_relation,
                &self.ways_map,
                self.show_only,
                &mut self.building_parts,
            );
            if way_from_relation.is_some() {
                self.ways_map
                    .insert(osm_relation.id, way_from_relation.unwrap());
                self.parts.push(osm_relation.id);
            }
        }

        println!("scan: part len = {:?}", self.parts.len());
        for part_id in &self.parts {
            println!("part_id: {part_id}");
            //if *part_id != 278033600 {
            //    continue;
            //}
            let part = &self.ways_map.get(part_id).unwrap();
            let shape = part.footprint.clone();
            let positions = part.footprint.positions.clone(); // todo: avoid clone! how? by ref clashes with ownership
            for building_id in &self.buildings {
                // println!("building_id: {building_id}");
                //if *building_id != 239592652 {
                //    continue;
                //}
                let building = self.ways_map.get_mut(building_id).unwrap();
                if *part_id == 278033600 && *part_id == 1124726437 {
                    println!("part = {}", part_id);
                };
                if building.footprint.substract_done(&positions) {
                    if building.footprint.positions.is_empty() {
                        println!(
                            "??? push hole to mepty3! part: {} building: {} len: {}",
                            part_id,
                            building_id,
                            building.footprint.positions.len()
                        );
                    }

                    //println!(
                    //    "footprint.holes: part = {} building: {} f:{}",
                    //    part_id,
                    //    building_id,
                    //    building.footprint.positions.len()
                    //);
                    building.footprint.holes.push(shape.clone());
                };
            }
        }

        println!("scan:: part len = {:?}", self.parts.len());
        for part_id in &self.parts {
            println!("part_id:: {part_id}");
            let part = self.ways_map.get_mut(part_id).unwrap();
            way(*part_id, part, self.show_only, &mut self.building_parts);
        }

        println!("scan: way len = {:?}", self.buildings.len());
        for building_id in &self.buildings {
            println!("building_id:: {building_id}");
            let building = self.ways_map.get_mut(building_id).unwrap();
            way(
                *building_id,
                building,
                self.show_only,
                &mut self.building_parts,
            );
        }
    }
}

fn way(id: u64, osm_way: &mut OtbWay, show_only: u64, building_parts: &mut Vec<BuildingPart>) {
    // todo: Fight Rust and make this {} a fn

    //println!("scan: way id = {:?}", id);
    if show_only > 0 && id != show_only {
        return;
    }

    if osm_way.tags.is_none() {
        return;
    }

    let tags = osm_way.tags.as_ref().unwrap();

    let part_option = tags_get2(tags, "building:part", "building");
    if part_option.is_none() && show_only == 0 {
        //println!("Unprocessed relation non-part tag {}", element.id);
        return;
    }

    // ??? not only parts!    || show_only < 0
    let string_no = &NO.to_string();
    let part = tags.get("building:part").unwrap_or(string_no);
    if part != NO || show_only > 0 || true {
        building_parts.push(building(&mut osm_way.footprint, id, tags));
    }
}
fn relation(
    id: u64,
    osm_relation: &OtbRelation,
    ways_map: &HashMap<u64, OtbWay>,
    show_only: u64,
    _building_parts: &mut Vec<BuildingPart>,
) -> Option<OtbWay> {
    // println!("scan: rel. id = {:?}", id);
    if show_only > 0 && id != show_only {
        return None;
    }

    if osm_relation.tags.is_none() {
        return None;
    }

    let string_no = &NO.to_string();
    let tags = osm_relation.tags.as_ref().unwrap();
    let part = tags.get("building:part").unwrap_or(string_no);
    if part == NO && show_only > 0 {
        return None;
    }

    // todo: process relation type building? The outer and parts are processed by the normal code anyway, are they?
    // except the outer has no tags!

    /******* self.relation(*id, osm_relation); //  ****/

    println!("Relation, id: {:?}", id);

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
    if part_option.is_none() && show_only == 0 {
        //println!("Unprocessed relation non-part tag {}", element.id);
        return None;
    }

    let mut footprint = Footprint::new(id);

    // first scann for outer, later vo inner
    for member in &members {
        // println!("mem: {:?}", &member);
        if member.relation_type != "way" {
            return None;
        }
        match member.role.as_str() {
            "outer" => {
                let outer_ref = member.reference;
                let option = ways_map.get(&outer_ref);
                if option.is_none() {
                    println!("outer none, id/ref: {}", outer_ref);
                    return None;
                }
                // Todo: cloning footprint twice can't be the solution
                footprint = ways_map.get(&outer_ref).unwrap().footprint.clone();
            }
            _ => (),
        }
    }

    for member in &members {
        //println!("mem: {:?}", &member);
        match member.role.as_str() {
            "inner" => {
                inner(member.reference, ways_map, &mut footprint);
            }
            _ => (),
        }
    }

    // building_parts.push...
    Some(OtbWay {
        _id: id,
        footprint,
        tags: Some(tags.clone()),
    })
}

fn inner(elements_ref: u64, ways_map: &HashMap<u64, OtbWay>, footprint: &mut Footprint) {
    //println!("elements_ref: {:?}", &elements_ref);
    let option = ways_map.get(&elements_ref);
    if option.is_none() {
        println!("inner none, id/ref: {}", elements_ref);
        return;
    }
    let hole = ways_map.get(&elements_ref).unwrap().footprint.clone();
    footprint.push_hole(hole);
    //println!("inner way; {:?}", &elements_ref);
}
