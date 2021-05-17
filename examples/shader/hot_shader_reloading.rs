use bevy::{
    asset::LoadState,
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::shape,
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        renderer::RenderResources,
        shader::ShaderStages,
        texture::{self, AddressMode},
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    Setup,
    Finished,
}

/// This example illustrates how to load shaders such that they can be
/// edited while the example is still running.
fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_state(AppState::Setup)
        .add_system_set(
            SystemSet::on_enter(AppState::Setup).with_system(load_terrain_assets.system()),
        )
        .add_system_set(
            SystemSet::on_update(AppState::Setup).with_system(check_terrain_assets.system()),
        )
        .add_system_set(SystemSet::on_enter(AppState::Finished).with_system(setup.system()))
        .add_system(rotator_system.system())
        .run();
}

struct TerrainAssets {
    texture: Handle<Texture>,
    vs: Handle<Shader>,
    fs: Handle<Shader>,
}

impl TerrainAssets {
    fn as_vec(&self) -> Vec<HandleUntyped> {
        vec![
            self.texture.clone_untyped(),
            self.vs.clone_untyped(),
            self.fs.clone_untyped(),
        ]
    }
}

fn load_terrain_assets(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    asset_server.watch_for_changes().unwrap();

    let terrain_assets = TerrainAssets {
        texture: asset_server.load("textures/terrain_LUT.png"),
        vs: asset_server.load("shaders/hot.vert"),
        fs: asset_server.load("shaders/hot.frag"),
    };
    commands.insert_resource(terrain_assets);
}

fn check_terrain_assets(
    terrain_assets: Res<TerrainAssets>,
    mut state: ResMut<State<AppState>>,
    asset_server: Res<AssetServer>,
) {
    match asset_server
        .get_group_load_state(terrain_assets.as_vec().into_iter().map(|handle| handle.id))
    {
        LoadState::Loaded => state.set(AppState::Finished).unwrap(),
        LoadState::Loading | LoadState::NotLoaded => {}
        LoadState::Failed => panic!(),
    }
}

fn setup(
    mut commands: Commands,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut textures: ResMut<Assets<Texture>>,
    terrain_assets: Res<TerrainAssets>,
) {
    // Create a new shader pipeline with shaders loaded from the asset directory
    let mut pipe = PipelineDescriptor::default_config(ShaderStages {
        vertex: terrain_assets.vs.clone(),
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

    // cube
    // commands.spawn_bundle(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::Torus {
    //         radius: 10.0,
    //         ring_radius: 5.0,
    //         subdivisions_segments: 100,
    //         subdivisions_sides: 10,
    //     })),
    //     render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
    //         pipeline_handle,
    //     )]),
    //     transform: Transform::from_xyz(0.0, 0.0, 0.0),
    //     material,
    //     ..Default::default()
    // });

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Icosphere {
            radius: 10.0,
            subdivisions: 10,
        })),
        render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
            pipeline_handle,
        )]),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        material,
        ..Default::default()
    });

    // light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 20000.0,
            range: 2000.0,
            ..Default::default()
        },
        transform: Transform::from_xyz(0.0, 20.0, 50.0),
        ..Default::default()
    });

    // camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(50.0, 25.0, -50.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(Rotates);
}

/// this component indicates what entities should rotate
struct Rotates;

fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<Rotates>>) {
    for mut transform in query.iter_mut() {
        *transform = Transform::from_rotation(
            Quat::from_rotation_y((4.0 * std::f32::consts::PI / 20.0) * time.delta_seconds())
                * Quat::from_rotation_z((4.0 * std::f32::consts::PI / 40.0) * time.delta_seconds()),
        ) * *transform;
    }
}
