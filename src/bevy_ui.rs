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
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_indices(Indices::U32(osm_mesh.indices_to_vertices.clone()))
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, osm_mesh.vertices_colors.clone())
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        osm_mesh.vertices_positions.clone(),
    );

    let mesh_handle = meshes.add(mesh);
    commands.spawn((
        PbrBundle {
            mesh: mesh_handle,
            material: materials.add(Color::srgb(1.0, 1.0, 1.0)),
            ..default()
        },
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
            transform.rotate_x(time.delta_seconds() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        for mut transform in &mut query {
            transform.rotate_x(-time.delta_seconds() / 1.2);
        }
    }

    if keyboard_input.pressed(KeyCode::ArrowLeft /* KeyY Z in German */) {
        for mut transform in &mut query {
            transform.rotate_y(time.delta_seconds() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        // KeyU
        for mut transform in &mut query {
            transform.rotate_y(-time.delta_seconds() / 1.2);
        }
    }

    if keyboard_input.pressed(KeyCode::KeyZ /*  Y in German */) {
        for mut transform in &mut query {
            transform.rotate_z(time.delta_seconds() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        for mut transform in &mut query {
            transform.rotate_z(-time.delta_seconds() / 1.2);
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
    // Transform for the camera and lighting, looking at (0,0,0) (the position of the mesh).
    let s = osm_meshes.scale as f32;
    let camera_transform = Transform::from_xyz(0. * s, 20. * s, 30. * s)
        .looking_at(Vec3::new(0., s * 5., 0.), Vec3::Y);

    // Camera in 3D space.
    commands.spawn(Camera3dBundle {
        transform: camera_transform,
        ..default()
    });

    // camera
    //commands.spawn((Camera3d::default(), camera_transform));

    // Todo: https://bevyengine.org/examples/camera/camera-orbit/

    // Light - ??? No reaction!
    let s = 50.;
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            // https://bevyengine.org/examples/3d-rendering/shadow-caster-receiver/
            intensity: 1_000_000.0 * 10.,
            ..default()
        },
        transform: Transform::from_xyz(s * 4., s * 5., s * 40.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    for mesh in &osm_meshes.vec {
        spawn_osm_mesh(mesh, &mut commands, &mut meshes, &mut materials);
    }
}

pub fn bevy_init(osm_meshes: Vec<OsmMeshAttributes>, scale: f64) {
    // BEVY-App
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::srgb(0.5, 0.5, 0.5)))
        .insert_resource(OsmMeshes {
            vec: osm_meshes,
            scale,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, input_handler)
        .run();
}
