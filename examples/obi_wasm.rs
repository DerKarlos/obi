// Bevy alowes build vor wasm by default. How ever they do that, thank you. No async/tokio needed?
// But the bevy_web_asset does not always work. See below

// Usefull info for (Custom-) asset: https://taintedcoders.com/bevy/assets

const LOCAL_TEST: bool = false;
// Test with native build and local files runs well. Not with web files. See C) below
// Test with wasm build and local files runs well.

use bevy::{
    asset::{AssetLoader, AssetMetaCheck, LoadContext, io::Reader},
    prelude::{
        App, Asset, AssetApp, AssetPlugin, AssetServer, Assets, Commands, Component,
        DefaultPlugins, Handle, Mesh, PluginGroup, Query, Res, ResMut, Resource, StandardMaterial,
        Startup, Text, Update, Window, WindowPlugin, With, default, info,
    },
    reflect::TypePath,
};
// Time,Timer, TimerMode,

use bevy_args::{BevyArgsPlugin, Deserialize, Parser, Serialize}; // https://github.com/mosure/bevy_args

use bevy_web_asset::WebAssetPlugin; // https://github.com/johanhelsing/bevy_web_asset
/*
 * Bevy not only loads from files, but from the web. THis crate adds http(s)
 * The bevy ability to read the extention and create a bevy/rust type is kept.
 * But json is not part of the bevy extentions, a custom asset loade is used.
 * It does not deserialize the json, cause it would need the rust data structures.
 * That structures shall be keep inside the OSM-Toolbox. So, only a vec/string is loaded.
 *
 * bevy_web_asset does not always work!
 * A) Bevy tries to load the rust data structure from an .meta file and causes load/log errors like: http://localhost:3000/assets/bbox.json.meta
 * B) Bevy quests the crate to add the .meta to the url. If the url includes parameter this results in an illegal url? Not accroding to the log. But it seems to cause a different error code as 404 and the download is broken.
 *    Luckily, there is a DefaultPlugins-option meta_check = AssetMetaCheck::Never to avoid this error B) and A).
 *    SEE: https://github.com/johanhelsing/bevy_web_asset/issues/20
 * C) Building native, loading draws: ERROR bevy_asset::server: Encountered an I/O error while loading asset: unexpected status code 500 while loading https://api.openstreetmap.org/api/0.6/way/121486088/full.json?: invalid HTTP version
 *    SEE: https://github.com/johanhelsing/bevy_web_asset/issues/44
 * Branching and investigatin the crate is not easy. How to log the http-trafic? May be this:
 * https://medium.com/@jpmtech/getting-started-with-instruments-a35485574601
 */

use thiserror::Error;

#[derive(Asset, TypePath, Debug)]
struct OsmApiAsset {
    // todo: As from_slice(&bytes is slow, use String
    bytes: Vec<u8>,
}

#[derive(Default)]
struct OsmApiAssetLoader;

#[derive(Component)]
struct TextUI;

/*

// timer
#[derive(Resource, Default)]
pub struct WatchDogTime(Timer);

impl WatchDogTime {
    pub fn new() -> Self {
        Self(Timer::from_seconds(14.0, TimerMode::Once))
    }
}
fn watch_dog(time: Res<Time>, mut watch_dog_time: ResMut<WatchDogTime>) {
    watch_dog_time.0.tick(time.delta());
    info!("{:?} delta {:?}", watch_dog_time.0, time.delta());
}

fn end_watch_dog(
    watch_dog_time: Res<WatchDogTime>,
    mut app_state: ResMut<AppState>,
    mut text_query: Query<&mut Text, With<TextUI>>,
) {
    if app_state.step1 && watch_dog_time.0.finished() {
        app_state.step1 = false;
        info!("fin {:?}", watch_dog_time.0.finished());
        let message = format!(
            "OBI - OSM Building Inspector\n!!a Way {:?} may not exist !!",
            app_state.way_id
        );
        for mut text in text_query.iter_mut() {
            text.0 = message.clone();
        }
        info!("{:?}", &message);
    }
}

*/

/// Possible errors that can be produced by [`OsmApiAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
enum OsmApiAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not read OSM api: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for OsmApiAssetLoader {
    type Asset = OsmApiAsset;
    type Settings = ();
    type Error = OsmApiAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        // if error 404, this load will not be called!
        // https://github.com/bevyengine/bevy/discussions/20371   discussion !!!

        info!("Loading Bytes...");
        let mut bytes = Vec::new();
        let result = reader.read_to_end(&mut bytes).await;
        match result {
            Ok(_) => Ok(OsmApiAsset { bytes }),
            Err(e) => {
                info!("Loading Error: {}", e);
                panic!("Problem loading the way: {e:?}");
                //Ok(OsmApiAsset { bytes })
            }
        }

        //Ok(OsmApiAsset { bytes })
    }
}

#[derive(Default, Debug, Resource, Serialize, Deserialize, Parser)]
#[command(about = "a minimal example of bevy_args", version, long_about = None)]
pub struct UrlCommandLineArgs {
    // ----------- St Paul's Cathedral: 369161987 - passau_dom_id: 24771505 - reifenberg_id: 121486088 - westminster_id: 367642719
    #[arg(short, long, default_value = "369161987")]
    pub way: u64,
    #[arg(short, long, default_value = "0")]
    pub only: i32,
    #[arg(short, long, default_value = "0")]
    pub range: i32,
}
// How to run:
// RUST_BACKTRACE=1 cargo run --example obi_wasm -- --way 139890029  // Error! in bevy_web_asset (html-lib)
// http://localhost:8080/?way=24771505

fn read_and_use_args(args: Res<UrlCommandLineArgs>, mut state: ResMut<AppState>) {
    info!("args: {:?}", *args);
    state.way_id = args.way as u64;
    state.show_only = args.only as u64;
    state.range = args.range as f32;
    state.way_only = if args.range > 0 { 0 } else { args.way };
}

#[derive(Resource, Default, Debug)]
struct AppState {
    // Strange!: The value api is never set like this: let api = InputJson::new(); // InputJson or InputLib
    // but it works!?!?!? Well, it's a struct with only a string, set with ::new() so:
    // Bevy seems to create and fill this struct State by default values.
    api: osm_tb::InputOsm, // InputJson only. InputLib does not support a splitted solution to read the API external and only scan the byte stream.
    way_id: u64,
    show_only: u64,
    way_only: u64,
    range: f32,
    asset: Handle<OsmApiAsset>,
    step1: bool,
    step2: bool,
    gpu_ground_null_coordinates: osm_tb::GeographicCoordinates,
}

fn on_load(
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    mut app_state: ResMut<AppState>,
    mut control_value: ResMut<osm_tb::ControlValues>,
    mut text_query: Query<&mut Text, With<TextUI>>,
    loaded_bytes: Res<Assets<OsmApiAsset>>,
    asset_server: Res<AssetServer>,
) {
    //for mut text in text_query.iter_mut() {
    //    text.0 = format!(
    //        "OBI - OSM Building Inspector\nTEST s1:{} s2:{}",
    //        app_state.step1, app_state.step2
    //    );
    //}

    let asset = loaded_bytes.get(&app_state.asset);

    if asset.is_none() {
        //bytes_assets.
        // info!("Bytes Not Ready");
        return;
    }

    if app_state.step2 {
        return;
    }

    if !app_state.step1 {
        info!(
            "Bytes Size: {} Bytes, range: {}",
            asset.unwrap().bytes.len(),
            app_state.range
        );

        let mut bounding_box = app_state
            .api
            .geo_bbox_of_way_vec(&asset.unwrap().bytes, app_state.way_id);
        bounding_box.min_range(app_state.range as f64);
        app_state.range = (bounding_box.max_radius() * osm_tb::LAT_FAKT) as f32;
        control_value.distance = app_state.range * 1.0;
        control_value.use_first_point_for_orientation = true;

        // load building
        app_state.gpu_ground_null_coordinates = bounding_box.center_as_geographic_coordinates();
        let mut url = app_state.api.bbox_url(&bounding_box);
        info!("**** bbox_url: {url}");

        if LOCAL_TEST {
            url = "bbox.json".into();
        }

        app_state.asset = asset_server.load(&url);
        app_state.step1 = true;

        for mut text in text_query.iter_mut() {
            text.0 = format!(
                "OBI - OSM Building Inspector\nWay {:?}, loading OSM tagging",
                app_state.way_id
            );
        }
    } else {
        // step2
        for mut text in text_query.iter_mut() {
            text.0 = format!(
                "OBI - OSM Building Inspector\nWay {:?}, calucating 3D",
                app_state.way_id
            );
        }
        let buildings_and_parts = app_state.api.scan_json_to_osm_vec(
            &asset.unwrap().bytes,
            &app_state.gpu_ground_null_coordinates,
            app_state.show_only,
            app_state.way_only,
        );
        info!(
            "json scan done, buildings: {:?} ",
            buildings_and_parts.len()
        );

        if buildings_and_parts.is_empty() {
            for mut text in text_query.iter_mut() {
                text.0 = format!(
                    "OBI - OSM Building Inspector\nWay {:?}, !! No Building(s) !!",
                    app_state.way_id
                );
            }
        } else {
            let osm_meshes = osm_tb::scan_elements_from_layer_to_mesh(buildings_and_parts);
            osm_tb::bevy_osm_load(commands, meshes, materials, osm_meshes, app_state.range);
            for mut text in text_query.iter_mut() {
                text.0 = "".into();
            }
        }

        app_state.step2 = true;
    }
}

fn setup(mut commands: Commands, mut state: ResMut<AppState>, asset_server: Res<AssetServer>) {
    // Get the geographic center of the GPU scene. Example: https://api.openstreetmap.org/api/0.6/way/121486088/full.json

    // Text-UI  (https://bevy.org/examples/ui-user-interface/text/)
    let text = format!("OBI - OSM Building Inspector\nWay {:?}", state.way_id);
    info!("OBI2 - OSM Building Inspector\nWay {:?}", state.way_id);
    println!("OBI3 - OSM Building Inspector\nWay {:?}", state.way_id);

    commands.spawn((
        (Text::new(text), TextUI),
        bevy::prelude::Node {
            position_type: bevy::prelude::PositionType::Relative,
            top: bevy::prelude::Val::Px(111.),
            left: bevy::prelude::Val::Px(111.),
            ..default()
        },
    ));

    let mut url = state.api.way_url(state.way_id);
    info!("= Way_URL: {url}");

    if LOCAL_TEST {
        url = "way.json".into();
    }

    state.asset = asset_server.load(url);
}

fn main() {
    // Outputs (info! println!) don't work in this main fn

    App::new()
        .add_plugins(WebAssetPlugin::default()) // for http(s)
        .add_plugins(
            DefaultPlugins
                // It seems like the macOS Safari does not run fullscreen with the old code in the old dir, but iPad doas! Hm?
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .add_plugins(BevyArgsPlugin::<UrlCommandLineArgs>::default())
        .init_resource::<AppState>()
        .init_resource::<osm_tb::ControlValues>()
        .init_asset::<OsmApiAsset>()
        .init_asset_loader::<OsmApiAssetLoader>()
        .add_systems(Startup, setup)
        .add_plugins(osm_tb::ControlWithCamera)
        .add_systems(Update, on_load)
        //.init_resource::<WatchDogTime>()
        //.add_systems(Update, (watch_dog, end_watch_dog.after(watch_dog)))
        //.add_systems(Update, watch_dog)
        .add_systems(Startup, read_and_use_args)
        .run();
}
