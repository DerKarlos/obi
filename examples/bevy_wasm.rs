// Bevy alowes build vor wasm by default. How ever they do that, thank you. No async/tokio needed?
// But the bevy_web_asset does not work. It is clonded and doted with printnl!. This example is postponed

// Usefull info for (Custom) asset: https://taintedcoders.com/bevy/assets

const LOCAL_TEST: bool = false;
// Test with native build and local files runs well. Not with web files. See C) below
// Test with wasm build and local files runs well.

use bevy::asset::AssetMetaCheck;
use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
    reflect::TypePath,
};

use bevy_web_asset::WebAssetPlugin;
// By this crate, Bevy not only loads from files, but from the web.
// The bevy ability to read the extention and create a bevy/rust type is kept
// But json is not part of the bevy extentions, a custom asset loade is used.
// But it does not deserialize the json, cause the needed rust data structure.
// shall be keep it inside the OSM-Toolbox. Only a vec/string is loaded.
//
// bevy_web_asset does not always work!
// A) The crate or bevy(!) seems to try to load the rust data structure from an .meta file and causes load/log errors: http://localhost:3000/assets/bbox.json.meta
// B) The crate or bevy(!) adds the .meta to the url? If the url includes parameter this results in an illegal url?
// C) Building native, loading draws: ERROR bevy_asset::server: Encountered an I/O error while loading asset: unexpected status code 500 while loading https://api.openstreetmap.org/api/0.6/way/121486088/full.json?: invalid HTTP version
// Seem like I need to branch and investigate the crate.
// https://medium.com/@jpmtech/getting-started-with-instruments-a35485574601

use thiserror::Error;

use osm_tb::*;

#[derive(Asset, TypePath, Debug)]
struct BytesVec {
    // todo: As from_slice(&bytes is slow, use String
    bytes: Vec<u8>,
}

#[derive(Default)]
struct BytesAssetLoader;

/// Possible errors that can be produced by [`BytesAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
enum BytesAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load file: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for BytesAssetLoader {
    type Asset = BytesVec;
    type Settings = ();
    type Error = BytesAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        info!("Loading Bytes...");
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(BytesVec { bytes })
    }
}

fn main() {
    // Outputs don't work before App:new

    App::new()
        .add_plugins(WebAssetPlugin::default()) // for http(s)
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        //  .insert_resource(AssetMetaCheck::Never)
        .init_resource::<State>()
        .init_asset::<BytesVec>()
        .init_asset_loader::<BytesAssetLoader>()
        .add_systems(Startup, setup)
        .add_systems(Update, osm_tb::input_handler)
        .add_systems(Update, on_load)
        .run();
    info!("### OSM-BI ###");
}

#[derive(Resource, Default)]
struct State {
    bytes: Handle<BytesVec>,
    step1: bool,
    step2: bool,
    gpu_ground_null_coordinates: GeographicCoordinates,
}

fn setup(mut state: ResMut<State>, asset_server: Res<AssetServer>) {
    // The input API
    // Get the center of the GPU scene
    // https://api.openstreetmap.org/api/0.6/way/121486088/full.json
    let mut url = way_url(121486088); // _westminster_id: 367642719  _reifenberg_id: 121486088  aaa
    info!("++++++++++ Way_URL: {url}");
    // https://github.com/johanhelsing/bevy_web_asset

    if LOCAL_TEST {
        url = "way.json".to_string();
    } // else Todo: this error rises: bevy_asset::server:
    //   Encountered an I/O error while loading asset: unexpected status code 500
    //   while loading https://api.openstreetmap.org/api/0.6/way/121486088/full.json: invalid HTTP version
    // May be caused/not solved by the used bevy_web_asset
    // May be bevy_http_client would help. A simple HTTP client Bevy Plugin for both native and WASM, but NOT! using Assets loading

    // Will use BytesAssetLoader instead of CustomAssetLoader thanks to type inference
    state.bytes = asset_server.load(url);
}

fn on_load(
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<State>,
    bytes_assets: Res<Assets<BytesVec>>,
    asset_server: Res<AssetServer>,
) {
    let bytes = bytes_assets.get(&state.bytes);

    if bytes.is_none() {
        info!("Bytes Not Ready");
        return;
    }

    if state.step2 {
        return;
    }

    info!("Bytes Size: {} Bytes", bytes.unwrap().bytes.len());
    if !state.step1 {
        // info!("Bytes asset loaded: {:?}", bytes.unwrap());

        let bounding_box = geo_bbox_of_way_vec(&bytes.unwrap().bytes);
        state.gpu_ground_null_coordinates = bounding_box.center_as_geographic_coordinates();
        let mut url = bbox_url(&bounding_box);
        info!("************* bbox_url: {url}");

        if LOCAL_TEST {
            url = "bbox.json".to_string();
        }

        state.bytes = asset_server.load(url);
        state.step1 = true;
    } else {
        let building_parts =
            scan_osm_vec(&bytes.unwrap().bytes, &state.gpu_ground_null_coordinates, 0);
        info!("scan done, buildings: {:?} ", building_parts.len());
        let osm_meshes = scan_objects(building_parts);
        bevy_osm(commands, meshes, materials, osm_meshes, 25.);

        state.step2 = true;
    }
}
