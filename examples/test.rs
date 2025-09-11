use bevy::{
    prelude::*,
    render::{mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
};

#[derive(Component)]
struct CustomUV;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, input_handler)
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Create and save a handle to the mesh.
    let cube_mesh_handle: Handle<Mesh> = meshes.add(create_cube_mesh());

    // Render the mesh with the custom texture, and add the marker.
    commands.spawn((
        Mesh3d(cube_mesh_handle),
        MeshMaterial3d(materials.add(StandardMaterial { ..default() })),
        CustomUV,
    ));

    // Transform for the camera and lighting, looking at (0,0,0) (the position of the mesh).
    let dist = 33.;
    let camera_and_light_transform =
        Transform::from_xyz(dist, -dist, dist).looking_at(Vec3::ZERO, Vec3::Y);
    // Camera in 3D space.
    commands.spawn((Camera3d::default(), camera_and_light_transform));
    // Light up the scene.
    commands.spawn((PointLight::default(), camera_and_light_transform));
}

fn input_handler(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<CustomUV>>,
    time: Res<Time>,
) {
    if keyboard_input.pressed(KeyCode::KeyX) {
        for mut transform in &mut query {
            transform.rotate_x(time.delta_secs() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::KeyY) {
        for mut transform in &mut query {
            transform.rotate_y(time.delta_secs() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::KeyZ) {
        for mut transform in &mut query {
            transform.rotate_z(time.delta_secs() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::KeyR) {
        for mut transform in &mut query {
            transform.look_to(Vec3::NEG_Z, Vec3::Y);
        }
    }
}

fn create_cube_mesh() -> Mesh {
    // Keep the mesh data accessible in future frames to be able to mutate it in toggle_texture.
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            /* 0 */ [-27.419066613139673, 0., -31.33583999984012],
            /* 1 */ [-13.03044527993139, 0., -28.8800880003555],
            /* 2 */ [-13.74374087546301, 0., -24.735312000082104],
            /* 3 */ [-2.5830298123234026, 0., -22.82404799985443],
            /* 4 */ [-1.9273667037013107, 0., -26.657688000439634],
            /* 5 */ [11.819976488099567, 0., -24.31305599988491],
            /* 6 */ [11.099457284650905, 0., -20.112720000541344],
            /* 7 */ [22.411457042288635, 0., -18.179232000053958],
            /* 8 */ [22.894210471500347, 0., -20.990568000535745],
            /* 9 */ [36.266863811952206, 0., -18.701495999840745],
            /*10 */ [27.706951004079286, 0., 31.33583999984012],
            /*11 */ [-36.266602361617025, 0., 20.390519999839967],
        ],
    )
    .with_inserted_indices(Indices::U32(vec![
        11, 0, 1, 3, 4, 5, 7, 8, 9, 9, 10, 11, 11, 1, 2, 3, 5, 6, 7, 9, 11, 11, 2, 3, 6, 7, 11, 11,
        3, 6,
    ]));
    mesh.compute_normals();
    mesh
}
