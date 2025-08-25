use crate::bevy_control::{ControlValues, ControlWithCamera};
use crate::kernel_in::PI;
use crate::kernel_out::OsmMeshAttributes;

///////////////////////////////////////////////////////////////////////////////////////////////////
// BEVY ///////////////////////////////////////////////////////////////////////////////////////////

use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy::render::{
    mesh::Indices, //VertexAttributeValues},
    render_asset::RenderAssetUsages,
    render_resource::PrimitiveTopology,
};

#[derive(Resource)]
struct StartingValues {
    pub osm_meshes: Vec<OsmMeshAttributes>,
    pub range: f32,
    //pub ui: Option<EntityCommands>,
}

fn spawn_osm_mesh(
    osm_mesh: &OsmMeshAttributes,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // println!("{:?}", osm_mesh.vertices_colors);
    // println!("p {:?} c {:?} i {:?}", osm_mesh.vertices_positions.len(), osm_mesh.vertices_colors.len(), osm_mesh.indices_to_vertices.len() );

    let count = osm_mesh.vertices_positions.len(); // mesh.count_vertices();
    let uvs: Vec<[f32; 2]> = vec![[0.; 2]];
    let uvs = uvs.repeat(count);

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        osm_mesh.vertices_positions.clone(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, osm_mesh.vertices_colors.clone())
    .with_inserted_indices(Indices::U32(osm_mesh.indices_to_vertices.clone()));
    mesh.compute_normals();

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(Color::srgb(1., 1., 1.))),
    ));
}

// examples like obi.rs have no Bevy code. They setup Bevy here:
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    starting_values: ResMut<StartingValues>,
) {
    for mesh in &starting_values.osm_meshes {
        spawn_osm_mesh(mesh, &mut commands, &mut meshes, &mut materials);
    }

    environment(commands, meshes, materials, starting_values.range as f32);
}

#[derive(Component)]
struct TextUI;

fn environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    range: f32,
    // starting_values: &mut StartingValues,
) {
    //let range = starting_values.range;

    // light
    commands.spawn((
        DirectionalLight {
            illuminance: 2000., // * range,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, PI / 4., -PI / 4.)),
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 7.0, // What's that ???
            maximum_distance: range * 2.,
            ..default()
        }
        .build(),
    ));

    // circular base
    const SLIGHTLY_BELOW_GROUND_0: f32 = -0.01;

    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(range * 2.0, range * 2.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(150, 255, 150))),
        Transform {
            translation: Vec3::new(0., SLIGHTLY_BELOW_GROUND_0, 0.),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            ..default()
        },
    ));
}

// BEVY-APP ///////////////
// examples like obi.rs have no Bevy code. They init Bevy here:
pub fn render_init(
    osm_meshes: Vec<OsmMeshAttributes>,
    range: f32,
    use_first_mouse_key_for_orientation: bool,
) {
    let starting_values = StartingValues { osm_meshes, range };

    let control_values = ControlValues {
        use_first_point_for_rotation: use_first_mouse_key_for_orientation,
        distance: range * 1.0,
        ..default()
    };

    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::srgb(0.5, 0.5, 1.0)))
        .insert_resource(starting_values)
        .add_systems(Startup, setup)
        .insert_resource(control_values)
        .add_plugins(ControlWithCamera)
        .run();
}

// BEVY-OSM mesh load //////////
// Only used from example obi_main.rs: Adds osm parts, light and camera.
// Bevy-App and asset loading is already done in the example code
pub fn bevy_osm_load(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    osm_meshes: Vec<OsmMeshAttributes>,
    range: f32,
) {
    // OSM meshes
    for mesh in &osm_meshes {
        spawn_osm_mesh(mesh, &mut commands, &mut meshes, &mut materials);
    }
    commands.insert_resource(ClearColor(Color::srgb(0.5, 0.5, 1.0)));

    environment(commands, meshes, materials, range);
}
