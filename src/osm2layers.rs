///////////////////////////////////////
// The tactics to hanlde OSM tagging //
///////////////////////////////////////

use csscolorparser::parse;
use std::collections::HashMap;

use crate::kernel_in::RenderColor;
use crate::kernel_in::{BuildingPart, RoofShape};
use crate::shape::Shape;

// This constands may come from a (3D-)render shema
pub static DEFAULT_WALL_COLOR: RenderColor = [0.5, 0.5, 0.5, 1.0]; // "grey" = RenderColor = [0.5, 0.5, 0.5, 1.0];
pub static DEFAULT_ROOF_COLOR: RenderColor = [1.0, 0.0, 0.0, 1.0]; //  "red"  = RenderColor = [1.0, 0.0, 0.0, 1.0];
pub static DEFAULT_WALL_HEIGHT: f32 = 6.0; // two floors with each 3 meters
pub static DEFAULT_ROOF_HEIGHT: f32 = 2.0;
pub static DEFAULT_MIN_HEIGHT: f32 = 2.0;

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

pub fn parse_height(height: Option<&String>) -> f32 {
    if height.is_none() {
        return 0.;
    }

    let mut hu = height.unwrap().clone();

    if hu.ends_with("m") {
        hu = hu.strip_suffix("m").unwrap().to_string();
        //hu = &hu.as_str().strip_suffix("m").unwrap().to_string();
    }

    match hu.as_str().trim().parse() {
        Ok(height) => height,

        Err(error) => {
            println!("Error! parse_height: {} for:{}:", error, hu);
            0.
        }
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
        if let Some(tag) = tags.get(option2) {
            Some(tag)
        } else {
            None
        }
    }
}

pub fn building(
    mut footprint: Shape,
    id: u64,
    tags: &HashMap<String, String>,
    building_parts: &mut Vec<BuildingPart>,
) {
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
        None => RoofShape::None,
    };

    // ** Colors and Materials **
    let building_color = parse_color(
        tags_get2(tags, "building:colour", "colour"),
        DEFAULT_WALL_COLOR,
    );
    let roof_color = parse_color(tags.get("roof:colour"), DEFAULT_ROOF_COLOR);

    println!("Part id: {} roof: {:?}", id, roof_shape);

    let default_roof_heigt = match roof_shape {
        RoofShape::Flat => 0.0,
        RoofShape::Skillion => 2.0, // todo: accroding to width
        RoofShape::Gabled => 2.0,
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

    // This is NOT valid. F4maps is NOT doing it. Test with Reifenberg. It came from the Java-Tool
    // if bounding_box_rotated.east_larger_than_nord() {
    //     roof_angle = circle_limit(roof_angle + f32::to_radians(90.));
    // This way is a good example: 363815745 beause it has many nodes on the longer side
    // println!( "### {}: east_larger_than_nord: {}", element.id, roof_angle.to_degrees() );
    //}
    // println!( "id: {} roof_shape: {:?} angle: {}", element.id, roof_shape, roof_angle.to_degrees() );

    let building_part = BuildingPart {
        _id: id,
        _part: true, // ??? not only parts!
        footprint,
        //center,
        // _bounding_box: bounding_box,
        bounding_box_rotated,
        wall_height,
        min_height,
        building_color,
        roof_shape,
        roof_height,
        roof_angle,
        roof_color,
    };

    building_parts.push(building_part);
}
