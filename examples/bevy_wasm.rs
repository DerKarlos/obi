// Bevy alowes build vor wasm by default. How ever they do that, thank you. No async/tokio needed?
// But the bevy_web_asset does not always work. See below

// Usefull info for (Custom-) asset: https://taintedcoders.com/bevy/assets

const LOCAL_TEST: bool = false;
// Test with native build and local files runs well. Not with web files. See C) below
// Test with wasm build and local files runs well.

use bevy::{
    asset::{AssetLoader, AssetMetaCheck, LoadContext, io::Reader},
    prelude::{
        App, Asset, AssetApp, AssetPlugin, AssetServer, Assets, Commands, DefaultPlugins, Handle,
        Mesh, PluginGroup, Res, ResMut, Resource, StandardMaterial, Startup, Update, default, info,
    },
    reflect::TypePath,
};

use bevy_args::{BevyArgsPlugin, Deserialize, Parser, Serialize}; // https://github.com/mosure/bevy_args

use bevy_web_asset::WebAssetPlugin; // https://github.com/johanhelsing/bevy_web_asset
// Bevy not only loads from files, but from the web. THis crate adds http(s)
// The bevy ability to read the extention and create a bevy/rust type is kept.
// But json is not part of the bevy extentions, a custom asset loade is used.
// It does not deserialize the json, cause it would need the rust data structures.
// That structures shall be keep inside the OSM-Toolbox. So, only a vec/string is loaded.
//
// bevy_web_asset does not always work!
// A) Bevy tries to load the rust data structure from an .meta file and causes load/log errors like: http://localhost:3000/assets/bbox.json.meta
// B) Bevy quests the crate to add the .meta to the url. If the url includes parameter this results in an illegal url? Not accroding to the log. But it seems to cause a different error code as 404 and the download is broken.
//    Luckily, there is a DefaultPlugins-option meta_check = AssetMetaCheck::Never to avoid this error B) and A).
//    SEE: https://github.com/johanhelsing/bevy_web_asset/issues/20
// C) Building native, loading draws: ERROR bevy_asset::server: Encountered an I/O error while loading asset: unexpected status code 500 while loading https://api.openstreetmap.org/api/0.6/way/121486088/full.json?: invalid HTTP version
//    SEE: https://github.com/johanhelsing/bevy_web_asset/issues/44
// Branching and investigatin the crate is not easy. How to log the http-trafic? May be this:
// https://medium.com/@jpmtech/getting-started-with-instruments-a35485574601

use thiserror::Error;

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

#[derive(Default, Debug, Resource, Serialize, Deserialize, Parser)]
#[command(about = "a minimal example of bevy_args", version, long_about = None)]
pub struct UrlCommandLineArgs {
    // passau_dom_id: 24771505 reifenberg_id: 121486088 westminster_id: 367642719 - St Paul's Cathedral: 369161987
    #[arg(short, long, default_value = "369161987")]
    pub way: u64,
    #[arg(short, long, default_value = "0")]
    pub only: i32,
    #[arg(short, long, default_value = "0")]
    pub range: i32,
}
// How to run:
// RUST_BACKTRACE=1 cargo run --example bevy_wasm -- --way 139890029  // Error! in bevy_web_asset (html-lib)
// http://localhost:8080/?way=24771505

fn read_and_use_args(args: Res<UrlCommandLineArgs>, mut state: ResMut<State>) {
    info!(" {:?}", *args);
    state.way_id = args.way as u64;
    state.show_only = args.only as u64;
    state.range = args.range as f32;
}

#[derive(Resource, Default, Debug)]
struct State {
    // Strange!: The value api is never set like this: let api = InputJson::new(); // InputJson or InputLib
    // but it works!?!?!? Well, it's a struct with only a string, set with ::new() so:
    // Bevy seems to create and fill this struct State by default values.
    api: osm_tb::InputOsm, // InputJson only. InputLib does not support a splitted solution to read the API external and only scan the byte stream.
    way_id: u64,
    show_only: u64,
    range: f32,
    bytes: Handle<BytesVec>,
    step1: bool,
    step2: bool,
    gpu_ground_null_coordinates: osm_tb::GeographicCoordinates,
}

fn on_load(
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<State>,
    mut control_value: ResMut<osm_tb::ControlValues>,
    bytes_assets: Res<Assets<BytesVec>>,
    asset_server: Res<AssetServer>,
) {
    let bytes = bytes_assets.get(&state.bytes);

    if bytes.is_none() {
        // info!("Bytes Not Ready");
        return;
    }

    if state.step2 {
        return;
    }

    if !state.step1 {
        info!(
            "Bytes Size: {} Bytes, range: {}",
            bytes.unwrap().bytes.len(),
            state.range
        );
        // info!("Bytes asset loaded: {:?}", bytes.unwrap());

        let mut bounding_box = state.api.geo_bbox_of_way_vec(&bytes.unwrap().bytes);
        bounding_box.min_range(state.range);
        state.range = bounding_box.max_radius() * osm_tb::LAT_FAKT as f32;
        control_value.distance = state.range * 1.0;

        // load building
        state.gpu_ground_null_coordinates = bounding_box.center_as_geographic_coordinates();
        let mut url = state.api.bbox_url(&bounding_box);
        info!("**** bbox_url: {url}");

        if LOCAL_TEST {
            url = "bbox.json".into();
        }

        state.bytes = asset_server.load(url);
        state.step1 = true;
    } else {
        // step2
        let buildings_and_parts = state.api.scan_json_to_osm_vec(
            &bytes.unwrap().bytes,
            &state.gpu_ground_null_coordinates,
            state.show_only,
        );
        info!(
            "json scan done, buildings: {:?} ",
            buildings_and_parts.len()
        );
        let osm_meshes = osm_tb::scan_elements_from_layer_to_mesh(buildings_and_parts);
        osm_tb::bevy_osm(commands, meshes, materials, osm_meshes, state.range);

        state.step2 = true;
    }
}

fn setup(mut state: ResMut<State>, asset_server: Res<AssetServer>) {
    // Get the geographic center of the GPU scene. Example: https://api.openstreetmap.org/api/0.6/way/121486088/full.json
    let mut url = state.api.way_url(state.way_id);
    info!("= Way_URL: {url}");

    if LOCAL_TEST {
        url = "way.json".into();
    }

    state.bytes = asset_server.load(url);
}

fn main() {
    // Outputs don't work before App:new

    App::new()
        .add_plugins(WebAssetPlugin::default()) // for http(s)
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_plugins(BevyArgsPlugin::<UrlCommandLineArgs>::default())
        .init_resource::<State>()
        .init_resource::<osm_tb::ControlValues>()
        .init_asset::<BytesVec>()
        .init_asset_loader::<BytesAssetLoader>()
        .add_systems(Startup, read_and_use_args)
        .add_systems(Startup, setup)
        .add_plugins(osm_tb::ControlWithCamera)
        .add_systems(Update, on_load)
        .run();

    info!("### OSM-BI ###");
}
