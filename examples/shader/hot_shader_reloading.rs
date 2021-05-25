use bevy::{
    asset::LoadState,
    log,
    pbr::render_graph::PBR_PIPELINE_HANDLE,
    prelude::*,
    render::{
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::{base, RenderGraph, RenderResourcesNode},
        renderer::RenderResources,
        shader::{ShaderStage, ShaderStages},
        texture::AddressMode,
    },
};

// Names for new RenderGraph Nodes
mod node {
    pub const TERRAIN_MATERIAL_NODE: &str = "TerrainMaterial_node";
}

// We need an AppState to track loading
// This is required to modify the Texture::sampler, but we might as well use it to finish loading everything
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    Setup,
    Finished,
}

fn main() {
    let mut app = App::build();

    app.add_plugins(DefaultPlugins)
        // Adds the state
        .add_state(AppState::Setup)
        // and the state-dependent systems
        .add_system_set(
            SystemSet::on_enter(AppState::Setup).with_system(load_terrain_assets.system()),
        )
        .add_system_set(
            SystemSet::on_update(AppState::Setup).with_system(check_terrain_assets.system()),
        )
        .add_system_set(SystemSet::on_enter(AppState::Finished).with_system(setup.system()))
        // Show that the TerrainMaterial is updated
        // .add_system(terrain_material_cycler_system.system())
        .add_startup_system(setup_render_graph.system())
        .run();
}

// Resources for tracking the loaded assets
struct TerrainAssets {
    mesh: Handle<Mesh>,
    texture: Handle<Texture>,
    // vs: Handle<Shader>,
    fs: Handle<Shader>,
}

impl TerrainAssets {
    // Needed to be able to do a single get_group_load_state, can be done differently of course
    fn as_vec(&self) -> Vec<HandleUntyped> {
        vec![
            self.mesh.clone_untyped(),
            self.texture.clone_untyped(),
            // self.vs.clone_untyped(),
            self.fs.clone_untyped(),
        ]
    }
}

// Initiate loading
fn load_terrain_assets(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    asset_server.watch_for_changes().unwrap();

    let terrain_assets = TerrainAssets {
        mesh: asset_server.load("models/example_quarry2_simplified_3d_mesh.glb#Mesh0/Primitive0"),
        texture: asset_server.load("textures/terrain_LUT.png"),
        // vs: asset_server.load("shaders/hot.vert"),
        fs: asset_server.load("shaders/hot.frag"),
    };
    commands.insert_resource(terrain_assets);
}

// Runs repeatedly until the assets finish loading
fn check_terrain_assets(
    terrain_assets: Res<TerrainAssets>,
    mut state: ResMut<State<AppState>>,
    asset_server: Res<AssetServer>,
) {
    match asset_server
        .get_group_load_state(terrain_assets.as_vec().into_iter().map(|handle| handle.id))
    {
        LoadState::Loaded => {
            log::info!("Finished loading");
            state.set(AppState::Finished).unwrap()
        }
        LoadState::Loading | LoadState::NotLoaded => {}
        LoadState::Failed => panic!(),
    }
}

// TerrainMaterial is used by the terrain vertex shader to scale and offset the UVs
// Currently not an Asset, but can easily be turned into one if it's desirable to reuse the
// same material on multiple meshes
#[derive(Debug, RenderResources)]
struct TerrainMaterial {
    scale: f32,
    offset: f32,
}

fn setup_render_graph(mut render_graph: ResMut<RenderGraph>) {
    render_graph.add_system_node(
        node::TERRAIN_MATERIAL_NODE,
        RenderResourcesNode::<TerrainMaterial>::new(true),
    );
    render_graph
        .add_node_edge(node::TERRAIN_MATERIAL_NODE, base::node::MAIN_PASS)
        .unwrap();
}

fn setup(
    mut commands: Commands,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut textures: ResMut<Assets<Texture>>,
    terrain_assets: Res<TerrainAssets>,
) {
    // Create a new shader pipeline with a custom vertex shader loaded from the asset directory
    // and the pbr fragment shader
    let mut pipe = PipelineDescriptor::default_config(ShaderStages {
        // vertex: terrain_assets.vs.clone(),
        vertex: shaders.add(Shader::from_glsl(
            ShaderStage::Vertex,
            "#version 450

        layout(location = 0) in vec3 Vertex_Position;
        layout(location = 1) in vec3 Vertex_Normal;

        layout(set = 0, binding = 0) uniform CameraViewProj {
            mat4 ViewProj;
        };

        layout(set = 2, binding = 0) uniform Transform {
            mat4 Model;
        };

        layout(set = 2, binding = 1) uniform TerrainMaterial_scale {
            float scale;
        };

        layout(set = 2, binding = 2) uniform TerrainMaterial_offset {
            float offset;
        };

        layout(location = 0) out vec3 v_WorldPosition;
        layout(location = 1) out vec3 v_WorldNormal;
        layout(location = 2) out vec2 v_Uv;

        void main() {
            vec4 world_position = Model * vec4(Vertex_Position, 1.0);
            v_WorldPosition = world_position.xyz;

            v_WorldNormal = mat3(Model) * Vertex_Normal;

            v_Uv = vec2(0.5, world_position.y * scale + offset);

            gl_Position = ViewProj * world_position;
        }
        ",
        )),
        fragment: Some(terrain_assets.fs.clone()),
    });
    pipe.primitive.cull_mode = None;
    let pipeline_handle = pipelines.add(pipe);

    let mut texture = textures.get_mut(terrain_assets.texture.clone()).unwrap();
    texture.sampler.address_mode_v = AddressMode::Repeat;

    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(terrain_assets.texture.clone()),
        roughness: 1.0,
        metallic: 0.0,
        ..Default::default()
    });

    commands
        .spawn_bundle(PbrBundle {
            mesh: terrain_assets.mesh.clone(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                pipeline_handle,
            )]),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            material,
            ..Default::default()
        })
        .insert(TerrainMaterial {
            scale: 1.0 / 6.0,
            offset: 0.0,
        });

    // light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 200000.0,
            range: 2000.0,
            ..Default::default()
        },
        transform: Transform::from_xyz(0.0, 20.0, 50.0),
        ..Default::default()
    });

    // camera
    let mut transform = Transform::from_xyz(0.0, 5.0, -50.0).looking_at(Vec3::ZERO, Vec3::Y);
    transform.rotate(Quat::from_axis_angle(Vec3::X, -std::f32::consts::PI / 4.0));
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform,
        ..Default::default()
    });
}

fn terrain_material_cycler_system(time: Res<Time>, mut query: Query<&mut TerrainMaterial>) {
    query.for_each_mut(|mut terrain_material| {
        terrain_material.offset += time.delta_seconds() / 10.0
    });
}
