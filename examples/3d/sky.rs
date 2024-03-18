// # Sky and Atmosphere rendering for Bevy

// This Sky and Atmosphere rendering implementation is based on Sébastien Hillaire, "A Scalable and Production Ready Sky and Atmosphere Rendering Technique", 2020 available here https://sebh.github.io/publications/egsr2020.pdf and their (MIT licensed) example code https://github.com/sebh/UnrealEngineSkyAtmosphere

// Hillaire uses the transmittance equation from Bruneton & Neyret, "Precomputed Atmospheric Scattering", 2008

// The UnrealEngineSkyAtmosphere example code includes Bruneton's implementation available at https://github.com/ebruneton/precomputed_atmospheric_scattering and extensively documented at https://ebruneton.github.io/precomputed_atmospheric_scattering/atmosphere/functions.glsl.html#transmittance, but reimplements it in HLSL. Both were used as references. The rest of Brunetons code is similarly documented here https://ebruneton.github.io/precomputed_atmospheric_scattering/.

use bevy::{
    app::AppExit,
    input::{keyboard::KeyboardInput, ButtonState},
    pbr::{CascadeShadowConfigBuilder, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

use bevy::pbr::{Sky, SkyPlugin};

/// sky rendering example
fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(bevy::log::LogPlugin {
        // Uncomment this to override the default log settings:
        level: bevy::log::Level::TRACE,
        // filter: "wgpu=warn,bevy_ecs=info".to_string(),
        ..default()
    }))
    .add_plugins(SkyPlugin)
    .add_systems(Startup, (setup_camera, setup_terrain_scene))
    .add_systems(Update, keyboard_events)
    .add_systems(Update, rotate);

    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-1.0, 0.1, 1.0)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            ..default()
        },
        Sky {},
    ));
}

fn setup_terrain_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Configure a properly scaled cascade shadow map for this scene (defaults are too large, mesh units are in km)
    let cascade_shadow_config = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 0.3,
        maximum_distance: 3.0,
        ..default()
    }
    .build();

    // Sun
    commands
        .spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: Color::srgb(0.98, 0.95, 0.82),
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 0.0)
                .looking_at(Vec3::new(-0.15, -0.05, 0.25), Vec3::Y),
            cascade_shadow_config,
            ..default()
        })
        .insert(Rotate);

    // Terrain
    commands.spawn(SceneBundle {
        scene: asset_server.load("models/terrain/Mountains.gltf#Scene0"),
        ..default()
    });
}

#[derive(Component)]
struct Rotate;

fn rotate(mut query: Query<&mut Transform, With<Rotate>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_z(time.delta_seconds() / 2.);
    }
}

fn keyboard_events(
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut time: ResMut<Time<Virtual>>,
    mut app_exit_events: ResMut<Events<AppExit>>,
) {
    for event in keyboard_input_events.read() {
        match event {
            KeyboardInput {
                key_code: KeyCode::Escape,
                state: ButtonState::Pressed,
                ..
            } => {
                app_exit_events.send(AppExit);
            }
            KeyboardInput {
                key_code: KeyCode::Space,
                state: ButtonState::Pressed,
                ..
            } => {
                if time.is_paused() {
                    time.unpause();
                } else {
                    time.pause();
                }
            }
            _ => {}
        }
    }
}
