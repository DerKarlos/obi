///////////////////////////////////////
// The tactics to hanlde OSM tagging //
///////////////////////////////////////

use csscolorparser::parse;

use crate::kernel_in::RenderColor;

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
