use crate::api_out::OsmMeshAttributes;

///////////////////////////////////////////////////////////////////////////////////////////////////
// BEVY ///////////////////////////////////////////////////////////////////////////////////////////

use bevy::prelude::*;
use bevy::render::{
    mesh::Indices, //VertexAttributeValues},
    render_asset::RenderAssetUsages,
    render_resource::PrimitiveTopology,
};

#[derive(Resource)]
struct OsmMeshes {
    vec: Vec<OsmMeshAttributes>,
    scale: f64,
}

// Define a "marker" component to mark the custom mesh. Marker components are often used in Bevy for
// filtering entities in queries with With, they're usually not queried directly since they don't contain information within them.
#[derive(Component)]
struct Controled;

pub fn spawn_osm_mesh(
    osm_mesh: &OsmMeshAttributes,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // println!("{:?}", osm_mesh.vertices_colors);
    // println!("p {:?} c {:?} i {:?}", osm_mesh.vertices_positions.len(), osm_mesh.vertices_colors.len(), osm_mesh.indices_to_vertices.len() );
    //let mut mesh = Mesh::from(Cuboid::default());

    let count = osm_mesh.vertices_positions.len(); // mesh.count_vertices();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    uvs.push([0.; 2]); // todo: allways 0. is ok?
    let uvs = uvs.repeat(count);

    let mut normals: Vec<[f32; 3]> = Vec::new();
    normals.push([1.; 3]); // todo: allways 1. is ok?
    let normals = normals.repeat(count);

    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        osm_mesh.vertices_positions.clone(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, osm_mesh.vertices_colors.clone())
    .with_inserted_indices(Indices::U32(osm_mesh.indices_to_vertices.clone()));

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(Color::srgb(1., 1., 1.))),
        Controled,
    ));
}

// System to receive input from the user,
// check out examples/input/ for more examples about user input.
fn input_handler(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Controled>>,
    time: Res<Time>,
) {
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        for mut transform in &mut query {
            transform.rotate_x(time.delta_secs() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        for mut transform in &mut query {
            transform.rotate_x(-time.delta_secs() / 1.2);
        }
    }

    if keyboard_input.pressed(KeyCode::ArrowLeft /* KeyY Z in German */) {
        for mut transform in &mut query {
            transform.rotate_y(time.delta_secs() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        // KeyU
        for mut transform in &mut query {
            transform.rotate_y(-time.delta_secs() / 1.2);
        }
    }

    if keyboard_input.pressed(KeyCode::KeyZ /*  Y in German */) {
        for mut transform in &mut query {
            transform.rotate_z(time.delta_secs() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        for mut transform in &mut query {
            transform.rotate_z(-time.delta_secs() / 1.2);
        }
    }

    if keyboard_input.pressed(KeyCode::KeyR) {
        for mut transform in &mut query {
            transform.look_to(Vec3::NEG_Z, Vec3::Y);
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    osm_meshes: Res<OsmMeshes>,
) {
    for mesh in &osm_meshes.vec {
        spawn_osm_mesh(mesh, &mut commands, &mut meshes, &mut materials);
    }

    let s = osm_meshes.scale as f32;
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(10.0 * s))),
        MeshMaterial3d(materials.add(Color::srgb_u8(150, 255, 150))),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // cube
    /*
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0 * s, 1.0 * s, 1.0 * s))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    )); */

    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: (2000000. * s),
            range: 100. * s,
            ..default()
        },
        Transform::from_xyz(4.0 * s, 8.0 * s, 4.0 * s),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5 * s, 4.5 * s, 9.0 * s).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

pub fn bevy_init(osm_meshes: Vec<OsmMeshAttributes>, scale: f64) {
    // BEVY-App
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::srgb(0.5, 0.5, 0.5)))
        .insert_resource(OsmMeshes {
            vec: osm_meshes,
            scale: scale,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, input_handler)
        .run();
}
