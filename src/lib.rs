//// Varionus input modules are possible (OSM-Json, Vector-Tile-File, Overtures)
//// This crate may get splitted in the included modules

mod input_osm_json;
mod input_osm_lib;

// Interfaces from input modules to renderer
mod kernel_in;
mod shape;

// 3D and 2D rendere are possible
mod osm2layers;
mod render_3d;

// Interface from an rederer to an output
mod kernel_out;

// Variouns outputs are possible (UI, create a GLB file)
mod bevy_ui;
//mod f4control;

pub use bevy_ui::*;
pub use input_osm_json::JsonData;
pub use input_osm_json::*;
pub use input_osm_lib::InputLib;
pub use kernel_in::GeographicCoordinates;
pub use kernel_in::LAT_FAKT; // todo: hide in lib by fn
pub use kernel_out::*;
pub use osm2layers::*;
pub use render_3d::*;
pub use shape::*;
