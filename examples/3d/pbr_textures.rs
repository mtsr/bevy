use std::f32::consts::PI;

use bevy::wgpu::{WgpuFeature, WgpuFeatures, WgpuLimits, WgpuOptions};
use bevy::{pbr::AmbientLight, prelude::*};

fn main() {
    App::build()
        .insert_resource(WgpuOptions {
            features: WgpuFeatures {
                features: vec![WgpuFeature::PushConstants],
            },
            limits: WgpuLimits {
                max_push_constant_size: 128,
                ..Default::default()
            },
            ..Default::default()
        })
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
) {
    let mut light_sphere = None;

    commands
        .spawn_scene(asset_server.load("models/FlightHelmet/FlightHelmet.gltf#Scene0"))
        // Add a rotating light with a sphere to show it's position
        .spawn((Transform::default(), GlobalTransform::default(), Rotator))
        .with_children(|parent| {
            parent
                .spawn(PointLightBundle {
                    transform: Transform::from_translation(Vec3::new(0.0, 0.7, 2.0)),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(PbrBundle {
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

                    // TODO figure out a better way to remove shadowcaster
                    light_sphere = parent.current_entity();
                });
        })
        .spawn(PerspectiveCameraBundle {
            transform: Transform::from_xyz(0.7, 0.7, 1.0)
                .looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
            ..Default::default()
        });

    if let Some(entity) = light_sphere {
        commands.remove::<ShadowCaster>(entity);
    }
}

/// this component indicates what entities should rotate
struct Rotator;

/// rotates the parent, which will result in the child also rotating
fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<Rotator>>) {
    for mut transform in query.iter_mut() {
        transform.rotation *= Quat::from_rotation_y((2.0 * PI / 20.0) * time.delta_seconds());
    }
}
