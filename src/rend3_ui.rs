use rend3::types::glam::*;

// use osm_tb::{InputOsm, scan_objects};

use crate::kernel_out::OsmMeshAttributes;

const SAMPLE_COUNT: rend3::types::SampleCount = rend3::types::SampleCount::One;

fn vertex(pos: [f32; 3]) -> Vec3 {
    Vec3::from(pos)
}

const S: f32 = 130.0;
fn create_ground() -> rend3::types::Mesh {
    let mut vertex_positions: Vec<Vec3> = Vec::new();
    vertex_positions.push(vertex([S, 0., S]));
    vertex_positions.push(vertex([-S, 0., S]));
    vertex_positions.push(vertex([-S, 0., -S]));
    vertex_positions.push(vertex([S, 0., -S]));

    let index_data: &[u32] = &[
        0, 2, 1, 2, 0, 3, // far
    ];

    rend3::types::MeshBuilder::new(vertex_positions, rend3::types::Handedness::Left)
        .with_indices(index_data.to_vec())
        .build()
        .unwrap()
}

pub struct ObiApp {
    osm_meshes: Vec<OsmMeshAttributes>,
    ground_handle: Option<rend3::types::ObjectHandle>,
    object_handle: Option<rend3::types::ObjectHandle>,
    directional_light_handle: Option<rend3::types::DirectionalLightHandle>,
    point_lights: Vec<rend3::types::PointLightHandle>,
}

impl Default for ObiApp {
    fn default() -> Self {
        Self {
            osm_meshes: Vec::new(),
            ground_handle: None,
            object_handle: None,
            directional_light_handle: None,
            point_lights: Vec::new(),
        }
    }
}

impl rend3_framework::App for ObiApp {
    const HANDEDNESS: rend3::types::Handedness = rend3::types::Handedness::Left;

    fn sample_count(&self) -> rend3::types::SampleCount {
        SAMPLE_COUNT
    }

    fn setup(&mut self, context: rend3_framework::SetupContext<'_>) {
        let meshes = &self.osm_meshes; // scan_objects(building_parts);
        println!("meshes len: {:?}", meshes.len());

        let vertex_positions = &meshes[0].vertices_positions;
        let mut rend_vertex_positions: Vec<Vec3> = Vec::new();
        for vertex_position in vertex_positions {
            let rend_vertex_position = Vec3 {
                x: vertex_position[0],
                y: vertex_position[1],
                z: vertex_position[2],
            };
            rend_vertex_positions.push(rend_vertex_position)
        }
        let mut colors: std::vec::Vec<[u8; 4]> = Vec::new();
        let vertices_colors = &meshes[0].vertices_colors;
        for vertices_color in vertices_colors {
            colors.push([
                (vertices_color[0] * 256.) as u8,
                (vertices_color[1] * 256.) as u8,
                (vertices_color[2] * 256.) as u8,
                (vertices_color[3] * 256.) as u8,
            ]);
        }

        let index_data = &meshes[0].indices_to_vertices.clone();
        let mesh =
            rend3::types::MeshBuilder::new(rend_vertex_positions, rend3::types::Handedness::Left)
                .with_indices(index_data.clone())
                .with_vertex_color_0(colors)
                .build()
                .unwrap();

        // Create mesh and calculate smooth normals based on vertices
        // let mesh = create_mesh();

        // Add mesh to renderer's world.
        //
        // All handles are refcounted, so we only need to hang onto the handle until we
        // make an object.
        let mesh_handle = context.renderer.add_mesh(mesh).unwrap();

        // Add PBR material with all defaults except a single color.
        let material = rend3_routine::pbr::PbrMaterial {
            albedo: rend3_routine::pbr::AlbedoComponent::ValueVertex {
                value: Vec4::new(1.0, 1.0, 1.0, 1.0),
                srgb: true,
            },
            ..rend3_routine::pbr::PbrMaterial::default()
        };
        let material_handle = context.renderer.add_material(material);

        // Combine the mesh and the material with a location to give an object.
        let object = rend3::types::Object {
            mesh_kind: rend3::types::ObjectMeshKind::Static(mesh_handle),
            material: material_handle,
            transform: Mat4::IDENTITY,
        };

        // ground //
        let material = rend3_routine::pbr::PbrMaterial {
            albedo: rend3_routine::pbr::AlbedoComponent::Value(Vec4::new(0.3, 1.0, 0.3, 1.0)),
            ..rend3_routine::pbr::PbrMaterial::default()
        };
        let material_handle = context.renderer.add_material(material);

        let mesh_handle = context.renderer.add_mesh(create_ground()).unwrap();

        let ground = rend3::types::Object {
            mesh_kind: rend3::types::ObjectMeshKind::Static(mesh_handle),
            material: material_handle,
            transform: Mat4::IDENTITY,
        };

        // Creating an object will hold onto both the mesh and the material
        // even if they are deleted.
        //
        // We need to keep the object handle alive.
        self.ground_handle = Some(context.renderer.add_object(ground));
        self.object_handle = Some(context.renderer.add_object(object));

        let x = 30.;
        let view_location = Vec3::new(3.0 * x, 3.0 * x, -5.0 * x);
        let view = Mat4::from_euler(EulerRot::XYZ, -0.35, 0.5, 0.0);
        let view = view * Mat4::from_translation(-view_location);

        // Set camera's location
        context.renderer.set_camera_data(rend3::types::Camera {
            projection: rend3::types::CameraProjection::Perspective {
                vfov: 60.0,
                near: 0.1,
            },
            view,
        });

        // Create a single directional light
        //
        // We need to keep the directional light handle alive.
        self.directional_light_handle = Some(context.renderer.add_directional_light(
            rend3::types::DirectionalLight {
                color: Vec3::ONE,
                intensity: 1.0,
                // Direction will be normalized
                direction: Vec3::new(1.0, -4.0, 2.0),
                distance: 400.0,
                resolution: 2048,
            },
        ));

        let lights = [
            // position, color
            (vec3(0.1, 1.2, -1.5), vec3(1.0, 0.0, 0.0)),
            (vec3(1.5, 1.2, -0.1), vec3(0.0, 1.0, 0.0)),
        ];

        for (position, color) in lights {
            self.point_lights
                .push(context.renderer.add_point_light(rend3::types::PointLight {
                    position,
                    color,
                    radius: 2.0,
                    intensity: 4.0,
                }));
        }
    }

    fn handle_redraw(&mut self, context: rend3_framework::RedrawContext<'_, ()>) {
        // Swap the instruction buffers so that our frame's changes can be processed.
        context.renderer.swap_instruction_buffers();
        // Evaluate our frame'S world-change instructions
        let mut eval_output = context.renderer.evaluate_instructions();

        // Lock the routines
        let pbr_routine = rend3_framework::lock(&context.routines.pbr);
        let tonemapping_routine = rend3_framework::lock(&context.routines.tonemapping);

        // Build a rendergraph
        let mut graph = rend3::graph::RenderGraph::new();

        // Import the surface texture into the render graph.
        let frame_handle = graph.add_imported_render_target(
            context.surface_texture,
            0..1,
            0..1,
            rend3::graph::ViewportRect::from_size(context.resolution),
        );
        // Add the default rendergraph without a skybox
        context.base_rendergraph.add_to_graph(
            &mut graph,
            rend3_routine::base::BaseRenderGraphInputs {
                eval_output: &eval_output,
                routines: rend3_routine::base::BaseRenderGraphRoutines {
                    pbr: &pbr_routine,
                    skybox: None,
                    tonemapping: &tonemapping_routine,
                },
                target: rend3_routine::base::OutputRenderTarget {
                    handle: frame_handle,
                    resolution: context.resolution,
                    samples: SAMPLE_COUNT,
                },
            },
            rend3_routine::base::BaseRenderGraphSettings {
                ambient_color: Vec4::ZERO,
                clear_color: Vec4::new(0.10, 0.05, 0.10, 1.0), // Nice scene-referred purple
            },
        );

        // Dispatch a render using the built up rendergraph!
        graph.execute(context.renderer, &mut eval_output);
    }
}

// was main_rend3
pub fn render_init(osm_meshes: Vec<OsmMeshAttributes>, _scale: f64) {
    let app = ObiApp {
        osm_meshes,
        ground_handle: None,
        object_handle: None,
        directional_light_handle: None,
        point_lights: Vec::new(),
        // Why not??? ..default()
    };
    rend3_framework::start(
        app,
        winit::window::WindowBuilder::new()
            .with_title("obi-example")
            .with_maximized(true),
    );
}
