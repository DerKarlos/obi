#[cfg(feature = "f4control")]
use crate::control::PlayerPlugin;
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
pub struct StartingValues {
    pub vec: Vec<OsmMeshAttributes>,
    pub range: f32,
}

// Define a "marker" component to mark the custom mesh. Marker components are often used in Bevy for
// filtering entities in queries with With, they're usually not queried directly since they don't contain information within them.
#[derive(Component)]
pub struct Controled;

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
        Controled,
    ));
}

// System to receive input from the user,
// check out examples/input/ for more examples about user input.
pub fn input_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Controled>>,
    time: Res<Time>,
) {
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        for mut transform in &mut query {
            transform.translation.z -= time.delta_secs() * 50.;
        }
    }
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        for mut transform in &mut query {
            transform.translation.z += time.delta_secs() * 50.;
        }
    }

    if keyboard_input.pressed(KeyCode::KeyS) {
        for mut transform in &mut query {
            transform.rotate_x(time.delta_secs() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::KeyX) {
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
    osm_meshes: Res<StartingValues>,
) {
    for mesh in &osm_meshes.vec {
        spawn_osm_mesh(mesh, &mut commands, &mut meshes, &mut materials);
    }

    let range = osm_meshes.range;

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

    light_and_camera(commands, range);
}

fn light_and_camera(mut commands: Commands, range: f32) {
    // light

    if false {
        commands.spawn((
            PointLight {
                shadows_enabled: true,
                intensity: (1000000. * range),
                //range: 100. * range,
                ..default()
            },
            Transform::from_xyz(10.0 * range, 20.0 * range, 10.0 * range),
        ));
    } else {
        commands.spawn((
            DirectionalLight {
                illuminance: 2000., // * range,
                shadows_enabled: true,
                ..default()
            },
            Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, PI / 4., -PI / 4.)),
            CascadeShadowConfigBuilder {
                first_cascade_far_bound: 7.0, // What's that ???
                maximum_distance: range * 2.0,
                ..default()
            }
            .build(),
        ));
    }

    #[cfg(not(feature = "f4control"))]
    {
        // camera
        commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(-0.25 * range, 0.35 * range, 0.90 * range)
                .looking_at(Vec3::new(0., range * 0.2, 0.), Vec3::Y),
        ));
    }
}

// examples like obi.rs have no Bevy code. They init Bevy herer:
pub fn render_init(osm_meshes: Vec<OsmMeshAttributes>, range: f32) {
    println!(" "); // distance between test outputs and Bevy outputs

    let starting_values = StartingValues {
        vec: osm_meshes,
        range,
    };

    // BEVY-App
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    #[cfg(feature = "f4control")]
    app.add_plugins(PlayerPlugin);

    app.insert_resource(ClearColor(Color::srgb(0.5, 0.5, 1.0)))
        .insert_resource(starting_values)
        .add_systems(Startup, setup);

    #[cfg(not(feature = "f4control"))]
    app.add_systems(Update, input_control);

    app.run();
}

// Only used from example bevy-main.rs: Adds osm parts, light and camera.
// Bevy Main and asset loading is already done in the example code
pub fn bevy_osm(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    osm_meshes: Vec<OsmMeshAttributes>,
    scale: f32,
) {
    for mesh in &osm_meshes {
        spawn_osm_mesh(mesh, &mut commands, &mut meshes, &mut materials);
    }

    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(scale * 15.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(150, 255, 150))),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));

    println!(" "); // distance between test outputs and Bevy outputs

    commands.insert_resource(ClearColor(Color::srgb(0.5, 0.5, 1.0)));

    light_and_camera(commands, scale);
}
