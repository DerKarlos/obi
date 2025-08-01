// Varionus input modules are possible (OSM-Json, Vector-Tile-File, Overtures)
// This crate may get splitted in the included modules

// Input-Modules, OSM and may be other
#[cfg(feature = "json")]
mod input_osm_json;
//     #[cfg(feature = "json")]
//     pub use input_osm_json::InputJson;
//     #[cfg(feature = "json")]
//     pub use input_osm_json::JsonData;
#[cfg(feature = "json")]
pub use input_osm_json::*;

#[cfg(feature = "xmllib")]
mod input_osm_lib;
// --- #[cfg(feature = "xmllib")]
// --- mod osm_api_json;
// --- #[cfg(feature = "xmllib")]
// --- pub use osm_api_json::OsmApiJson;

#[cfg(feature = "xmllib")]
pub use input_osm_lib::InputOsm;

// Sort OSM taggign to data layer, used by the input modules
mod osm2layers;
pub use osm2layers::*;
mod footprint;
pub use footprint::*;

// Interfaces from the input modules to renderer
mod kernel_in;
pub use kernel_in::BoundingBox;
pub use kernel_in::GeographicCoordinates;
pub use kernel_in::GroundPosition;
pub use kernel_in::LAT_FAKT; // todo: hide in lib by fn

// 3D and 2D rendere are possible
mod symbolic_3d;
pub use symbolic_3d::*;

// Interface from an rederer to an output
mod kernel_out;
pub use kernel_out::*;

// Variouns outputs are possible (UI, create a GLB file

// BEVY
#[cfg(feature = "bevy")]
mod bevy_ui;
#[cfg(feature = "bevy")]
pub use bevy_ui::*;

// F4-like CONTROL
#[cfg(feature = "bevy")]
mod control;
#[cfg(feature = "bevy")]
pub use control::ControlValues;
#[cfg(feature = "bevy")]
pub use control::ControlWithCamera;

// REND3
#[cfg(feature = "rend3")]
mod rend3_ui;
// #[cfg(feature = "rend2")]
// mod rend3_ui;
