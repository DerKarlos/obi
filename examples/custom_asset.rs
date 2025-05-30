//! Implements loader for a custom asset type.

use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
    reflect::TypePath,
};
use bevy_web_asset::WebAssetPlugin;
//use serde::Deserialize;
//use serde::*;

use thiserror::Error;

use osm_tb::*;

#[derive(Asset, TypePath, Debug)]
struct Blob {
    // json_data: JsonData,
    bytes: Vec<u8>,
}

#[derive(Default)]
struct BlobAssetLoader;

/// Possible errors that can be produced by [`BlobAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
enum BlobAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load file: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for BlobAssetLoader {
    type Asset = Blob;
    type Settings = ();
    type Error = BlobAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        info!("Loading Blob...");
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        Ok(Blob { bytes })
    }
}

fn main() {
    // Outputs don't work before App:new

    App::new()
        .add_plugins(WebAssetPlugin::default()) // for http(s)
        .add_plugins(DefaultPlugins)
        .init_resource::<State>()
        .init_asset::<Blob>()
        .init_asset_loader::<BlobAssetLoader>()
        .add_systems(Startup, setup)
        .add_systems(Update, osm_tb::input_handler)
        .add_systems(Update, on_load)
        .run();
    info!("### OSM-BI ###");
}

#[derive(Resource, Default)]
struct State {
    blob: Handle<Blob>,
    step1: bool,
    step2: bool,
    gpu_ground_null_coordinates: GeographicCoordinates,
}

fn setup(mut state: ResMut<State>, asset_server: Res<AssetServer>) {
    // The input API
    // Get the center of the GPU scene
    // https://api.openstreetmap.org/api/0.6/way/121486088/full.json
    let url = way_url(121486088);
    info!("************* Way_URL: {url}");
    // https://github.com/johanhelsing/bevy_web_asset

    // Will use BlobAssetLoader instead of CustomAssetLoader thanks to type inference
    state.blob = asset_server.load("way.json");
}

fn on_load(
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<State>,
    blob_assets: Res<Assets<Blob>>,
    asset_server: Res<AssetServer>,
) {
    let blob = blob_assets.get(&state.blob);

    if blob.is_none() {
        info!("Blob Not Ready");
        return;
    }

    if state.step2 {
        return;
    }

    info!("Blob Size: {} Bytes", blob.unwrap().bytes.len());
    if !state.step1 {
        // info!("Blob asset loaded: {:?}", blob.unwrap());

        let bounding_box = geo_bbox_of_way_vec(&blob.unwrap().bytes);
        info!("bounding_box: {:?} ", bounding_box);
        state.gpu_ground_null_coordinates = bounding_box.center_as_geographic_coordinates();
        let url = bbox_url(&bounding_box);
        info!("************* bbox_url: {url}");
        state.blob = asset_server.load("bbox.json");
        state.step1 = true;
    } else {
        let building_parts =
            scan_osm_vec(&blob.unwrap().bytes, &state.gpu_ground_null_coordinates, 0);
        info!("scan done, buildings: {:?} ", building_parts.len());
        let osm_meshes = scan_objects(building_parts);
        bevy_osm(commands, meshes, materials, osm_meshes, 15.);

        state.step2 = true;
    }
}
