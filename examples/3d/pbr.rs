use bevy::prelude::*;

/// This example shows how to configure Physically Based Rendering (PBR) parameters.
fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .run();
}

/// set up a simple 3D scene
fn setup(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // add entities to the world
    for y in -2..=2 {
        for x in -5..=5 {
            let x01 = (x + 5) as f32 / 10.0;
            let y01 = (y + 2) as f32 / 4.0;
            commands
                // spheres
                .spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Icosphere {
                        radius: 0.45,
                        subdivisions: 32,
                    })),
                    material: materials.add(StandardMaterial {
                        albedo: Color::rgb(0.2, 0.2, 1.0),
                        pbr: Vec2::new(x01, y01), // vary key PBR parameters on a grid of spheres to show the effect
                        ..Default::default()
                    }),
                    transform: Transform::from_translation(Vec3::new(x as f32, y as f32, 0.0)),
                    ..Default::default()
                });
        }
    }
    commands
        // light
        .spawn(LightBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 1000.0, 1000.0)),
            ..Default::default()
        })
        // camera
        .spawn(PerspectiveCameraBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 8.0))
                .looking_at(Vec3::default(), Vec3::unit_y()),
            ..Default::default()
        });
}
