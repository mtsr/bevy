use std::f32::consts::PI;

use bevy::{
    pbr::{render_graph::PBR_PIPELINE_HANDLE, AmbientLight},
    prelude::*,
    render::{pipeline::PipelineDescriptor, shader::ShaderStages},
};

fn main() {
    App::build()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(rotator_system.system())
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
) {
    asset_server.watch_for_changes().unwrap();

    // Create a new shader pipeline with shaders loaded from the asset directory
    pipelines.set_untracked(
        PBR_PIPELINE_HANDLE,
        PipelineDescriptor::default_config(ShaderStages {
            vertex: asset_server
                .load::<Shader, _>("../crates/bevy_pbr/src/render_graph/pbr_pipeline/pbr.vert"),
            fragment: Some(
                asset_server
                    .load::<Shader, _>("../crates/bevy_pbr/src/render_graph/pbr_pipeline/pbr.frag"),
            ),
        }),
    );

    commands.spawn_scene(
        asset_server
            .load("models/GltfTestModels/NormalMirrorTest/NormalTangentMirrorTest.gltf#Scene0"),
    );

    commands
        // Add a rotating light with a sphere to show it's position
        .spawn_bundle((Transform::default(), GlobalTransform::default(), Rotates))
        .with_children(|parent| {
            parent
                .spawn_bundle(LightBundle {
                    transform: Transform::from_translation(Vec3::new(-4.0, 6.0, 3.0)),
                    light: Light {
                        range: 50.0,
                        intensity: 500.0,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Icosphere {
                            radius: 0.05,
                            subdivisions: 32,
                        })),
                        material: materials.add(StandardMaterial {
                            base_color: Color::YELLOW,
                            emissive: Color::WHITE * 10.0f32,
                            ..Default::default()
                        }),
                        transform: Transform::default(),
                        ..Default::default()
                    });
                });
        });

    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.0, 0.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    // commands
    //     .spawn_bundle(OrthographicCameraBundle::new_3d())
    //     .insert(Transform::from_xyz(0.0, 0.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y));
}

/// this component indicates what entities should rotate
struct Rotates;

/// rotates the parent, which will result in the child also rotating
fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<Rotates>>) {
    // for mut transform in query.iter_mut() {
    //     transform.rotation *= Quat::from_rotation_y((2.0 * PI / 20.0) * time.delta_seconds());
    // }
}
