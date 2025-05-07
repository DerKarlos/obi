///////////////////////////////////////
// The tactics to hanlde OSM tagging //
///////////////////////////////////////

use csscolorparser::parse;

use crate::internal_api_in::{BoundingBox, GroundPosition, RenderColor};

// This constands may come from a (3D-)render shema
pub static DEFAULT_WALL_COLOR: &str = "grey"; // RenderColor = [0.5, 0.5, 0.5, 1.0]; // "grey"
pub static DEFAULT_ROOF_COLOR: &str = "red"; // RenderColor = [1.0, 0.0, 0.0, 1.0]; // "red"
pub static DEFAULT_WALL_HEIGHT: f32 = 2.0 * 3.0; // two floors with each 3 meters
pub static DEFAULT_ROOF_HEIGHT: f32 = 0.0;

pub fn circle_limit(angle: f32) -> f32 {
    if angle > f32::to_radians(180.) {
        angle - f32::to_radians(360.)
    } else if angle < f32::to_radians(180.) {
        angle + f32::to_radians(360.)
    } else {
        angle
    }
}

pub fn parse_building_roof_rotation(
    footprint: &Vec<GroundPosition>,
) -> (f32, GroundPosition, BoundingBox, BoundingBox, bool) {
    let mut roof_angle = 0.;
    let mut longest_distance = 0.;
    let mut sum_north = 0.;
    let mut sum_east = 0.;
    let mut clockwise_sum = 0.;
    let mut bounding_box = BoundingBox::new();

    let mut last_position = footprint.last().unwrap();
    for position in footprint {
        bounding_box.include(position);
        // center
        sum_north += position.north;
        sum_east += position.east;
        // angle
        let (distance, angle) = position.distance_angle_to_other(last_position);
        if longest_distance < distance {
            longest_distance = distance;
            roof_angle = angle;
        }

        // direction
        clockwise_sum +=
            (position.north - last_position.north) * (position.east + last_position.east);

        last_position = position;
    }

    let count = footprint.len() as f32;
    let center = GroundPosition {
        north: sum_north / count,
        east: sum_east / count,
    };

    let mut bounding_box_rotated = BoundingBox::new();
    for position in footprint {
        let rotated_position = position.rotate_around_center(roof_angle, center);
        bounding_box_rotated.include(&rotated_position);
    }

    // If the shape is taller than it is wide after rotation, we are off by 90 degrees.
    if bounding_box_rotated.east_larger_than_nord() {
        roof_angle = circle_limit(roof_angle + f32::to_radians(90.));
    }

    let is_clockwise = clockwise_sum > 0.0;

    (
        roof_angle,
        center,
        bounding_box,
        bounding_box_rotated,
        is_clockwise,
    )
}

pub fn parse_color(color: &String) -> RenderColor {
    // Bevy pbr color needs f32, The parse has no .to_f32_array???}
    // https://docs.rs/csscolorparser/latest/csscolorparser/
    match parse(color.as_str()) {
        Ok(color_scc) => [
            color_scc.r as f32,
            color_scc.g as f32,
            color_scc.b as f32,
            color_scc.a as f32,
        ],

        Err(error) => {
            println!("parse_colour: {}", error);
            [0.5, 0.5, 1.0, 1.0] // "light blue?"
        }
    }
}

pub fn parse_height(height: Option<&String>, default: f32) -> f32 {
    if height.is_none() {
        return default;
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
            DEFAULT_ROOF_HEIGHT
        }
    }
}
