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
    let camera_and_light_transform =
        Transform::from_xyz(18., 18., 18.).looking_at(Vec3::ZERO, Vec3::Y);
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

// pos [[-7.109434, 4.0, -4.200336], [-6.5058937, 4.0, 5.089296], [7.109423, 4.0, 4.189224], [6.498721, 4.0, -5.089296], [6.498721, 0.0, 5.089296], [7.109423, 0.0, -4.189224], [6.498721, 4.0, 5.089296], [7.109423, 4.0, -4.189224], [7.109423, 0.0, -4.189224], [-6.5058937, 0.0, -5.089296], [7.109423, 4.0, -4.189224], [-6.5058937, 4.0, -5.089296], [-6.5058937, 0.0, -5.089296], [-7.109434, 0.0, 4.200336], [-6.5058937, 4.0, -5.089296], [-7.109434, 4.0, 4.200336], [-7.109434, 0.0, 4.200336], [6.498721, 0.0, 5.089296], [-7.109434, 4.0, 4.200336], [6.498721, 4.0, 5.089296]]
// ind [3, 0, 1, 1, 2, 3, 4, 5, 6, 5, 7, 6, 8, 9, 10, 9, 11, 10, 12, 13, 14, 13, 15, 14, 16, 17, 18, 17, 19, 18]

fn create_cube_mesh() -> Mesh {
    // Keep the mesh data accessible in future frames to be able to mutate it in toggle_texture.
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            /*0*/ [-7.1094340, 4.0, -4.200336],
            /*1*/ [-6.5058937, 4.0, 5.089296],
            /*2*/ [7.1094230, 4.0, 4.189224],
            [6.498721, 4.0, -5.089296],
            [6.498721, 0.0, 5.089296],
            [7.109423, 0.0, -4.189224],
            [6.498721, 4.0, 5.089296],
            [7.109423, 4.0, -4.189224],
            [7.109423, 0.0, -4.189224],
            [-6.5058937, 0.0, -5.089296],
            [7.109423, 4.0, -4.189224],
            [-6.5058937, 4.0, -5.089296],
            [-6.5058937, 0.0, -5.089296],
            [-7.109434, 0.0, 4.200336],
            [-6.5058937, 4.0, -5.089296],
            [-7.109434, 4.0, 4.200336],
            [-7.109434, 0.0, 4.200336],
            /*17*/ [6.498721, 0.0, 5.089296],
            /*18*/ [-7.109434, 4.0, 4.200336],
            /*19*/ [6.498721, 4.0, 5.089296],
        ],
    )
    .with_inserted_indices(Indices::U32(vec![
        /*d 3, 0, 1, */ 1, 2, 3,
        //1 4, 5, 6, 5, 7, 6,
        //2 8, 9, 10, 9, 11, 10,
        //3 12, 13, 14, 13, 15, 14,
        /*4 16, 17, 18, */
        17, 19, 18,
    ]))

    /* * /
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            // top (facing towards +y)
            [-0.5, 0.5, -0.5], // vertex with index 0
            [0.5, 0.5, -0.5],  // vertex with index 1
            [0.5, 0.5, 0.5],   // etc. until 23
            [-0.5, 0.5, 0.5],
            // bottom   (-y)
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
            [0.5, -0.5, 0.5],
            [-0.5, -0.5, 0.5],
            // right    (+x)
            [0.5, -0.5, -0.5],
            [0.5, -0.5, 0.5],
            [0.5, 0.5, 0.5], // This vertex is at the same position as vertex with index 2, but they'll have different UV and normal
            [0.5, 0.5, -0.5],
            // left     (-x)
            [-0.5, -0.5, -0.5],
            [-0.5, -0.5, 0.5],
            [-0.5, 0.5, 0.5],
            [-0.5, 0.5, -0.5],
            // back     (+z)
            [-0.5, -0.5, 0.5],
            [-0.5, 0.5, 0.5],
            [0.5, 0.5, 0.5],
            [0.5, -0.5, 0.5],
            // forward  (-z)
            [-0.5, -0.5, -0.5],
            [-0.5, 0.5, -0.5],
            [0.5, 0.5, -0.5],
            [0.5, -0.5, -0.5],
        ],
    )
    .with_inserted_indices(Indices::U32(vec![
        0, 3, 1, 1, 3, 2, // triangles making up the top (+y) facing side.
        4, 5, 7, 5, 6, 7, // bottom (-y)
        8, 11, 9, 9, 11, 10, // right (+x)
        12, 13, 15, 13, 14, 15, // left (-x)
        16, 19, 17, 17, 19, 18, // back (+z)
        20, 21, 23, 21, 22, 23, // forward (-z)
    ]))
    / * */
}
