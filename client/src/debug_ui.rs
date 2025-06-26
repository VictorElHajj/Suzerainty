use std::time::Duration;

use bevy::color::palettes;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use crate::states::SimulationState;
use crate::tectonics::TectonicsIteration;

#[derive(Copy, Clone)]
pub struct DebugUIPlugin {
    pub diagnostics: DebugDiagnostics,
}
impl Plugin for DebugUIPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.diagnostics);
        app.add_systems(PreStartup, setup)
            .add_systems(Update, update_fps)
            .add_systems(OnExit(SimulationState::MeshGen), add_mesh_gen_stats)
            .add_systems(OnExit(SimulationState::Tectonics), tectonics_add_time)
            .add_systems(
                Update,
                update_state_text.run_if(state_changed::<SimulationState>),
            )
            .add_systems(
                Update,
                update_tectonics.run_if(in_state(SimulationState::Tectonics)),
            );
    }
}

#[derive(Resource, Copy, Clone)]
pub struct DebugDiagnostics {
    pub seed: u64,
    pub subdivisions: Option<u32>,
    pub tiles: Option<usize>,
    pub mesh_gen_time: Option<Duration>,
    pub tectonics_time: Option<Duration>,
}

impl DebugDiagnostics {
    pub fn seed(seed: u64) -> Self {
        DebugDiagnostics {
            seed,
            subdivisions: None,
            tiles: None,
            mesh_gen_time: None,
            tectonics_time: None,
        }
    }
}

#[derive(Component)]
struct StateText;

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct SeedText;

#[derive(Component)]
struct SubdivisionsText;

#[derive(Component)]
struct TileAmountText;

#[derive(Component)]
struct MeshGenerationTimeText;

#[derive(Component)]
struct TectonicsParticleText;

#[derive(Component)]
struct TectonicsIterationText;

#[derive(Component)]
struct TectonicsTimeText;

fn update_fps(
    bevy_diagnostics: Res<DiagnosticsStore>,
    mut fps_text_query: Query<&mut Text, With<FpsText>>,
) {
    if let Some(fps) = bevy_diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps.smoothed() {
            // Update the value of the second section
            **fps_text_query.single_mut().unwrap() = format!("{value:.0}");
        }
    }
}

fn update_state_text(
    mut state_text_query: Query<&mut Text, With<StateText>>,
    current_state: Res<State<SimulationState>>,
) {
    **state_text_query.single_mut().unwrap() = current_state.to_string();
}

fn tectonics_add_time(
    diagnostics: Res<DebugDiagnostics>,
    mut tectonics_time_query: Query<&mut Text, With<TectonicsTimeText>>,
) {
    let tectonics_duration = diagnostics
        .tectonics_time
        .expect("Tectonics time should be set be set during Tectonics state");
    **tectonics_time_query.single_mut().unwrap() = format!(
        "{}.{}s",
        tectonics_duration.as_secs(),
        tectonics_duration.subsec_millis()
    );
}

fn add_mesh_gen_stats(
    diagnostics: Res<DebugDiagnostics>,
    mut texts: ParamSet<(
        Query<&mut Text, With<TileAmountText>>,
        Query<&mut Text, With<MeshGenerationTimeText>>,
        Query<&mut Text, With<SubdivisionsText>>,
    )>,
) {
    **texts.p0().single_mut().unwrap() = diagnostics
        .tiles
        .expect("Tiles should be set during MeshGen state")
        .to_string()
        // Thousands seperator
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(",");
    let mesh_gen_duration = diagnostics
        .mesh_gen_time
        .expect("Mesh generation time should be set during MeshGen state");
    **texts.p1().single_mut().unwrap() = format!(
        "{}.{}s",
        mesh_gen_duration.as_secs(),
        mesh_gen_duration.subsec_millis()
    );
    **texts.p2().single_mut().unwrap() = diagnostics
        .subdivisions
        .expect("Subdivisions should be set during MeshGen state")
        .to_string();
}

fn update_tectonics(
    diagnostics: Res<DebugDiagnostics>,
    tectonics_iteration: Res<TectonicsIteration>,
    mut texts: ParamSet<(
        Query<&mut Text, With<TectonicsParticleText>>,
        Query<&mut Text, With<TectonicsIterationText>>,
    )>,
) {
    **texts.p0().single_mut().unwrap() = diagnostics
        .tiles
        .expect("Tiles should be set during MeshGen state")
        .to_string()
        // Thousands seperator
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(",");
    **texts.p1().single_mut().unwrap() = tectonics_iteration
        .0
        .to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(",");
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    diagnostics: Res<DebugDiagnostics>,
) {
    commands.spawn((
        Node {
            width: Val::Px(200.),
            height: Val::Auto,
            margin: UiRect::with_left(UiRect::all(Val::Px(10.)), Val::Auto),
            padding: UiRect::all(Val::Px(10.)),
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        BackgroundColor(LinearRgba::new(0.01, 0.01, 0.01, 0.8).into()),
        children![
            (
                Node {
                    padding: UiRect::new(Val::Px(0.), Val::Px(0.), Val::Px(5.), Val::Px(5.)),
                    border: UiRect::bottom(Val::Px(1.)),
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                BorderColor(LinearRgba::new(0.2, 0.2, 0.2, 0.8).into()),
                children![
                    (
                        Node {
                            width: Val::Percent(100.),
                            ..Default::default()
                        },
                        children![
                            (
                                Text::new("FPS: "),
                                TextFont {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 12.0,
                                    ..default()
                                }
                            ),
                            (
                                Node {
                                    margin: UiRect::left(Val::Auto),
                                    ..Default::default()
                                },
                                Text::default(),
                                TextFont {
                                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                                    font_size: 12.0,
                                    ..Default::default()
                                },
                                TextColor(palettes::css::GOLD.into()),
                                FpsText
                            )
                        ]
                    ),
                    (
                        Node {
                            width: Val::Percent(100.),
                            ..Default::default()
                        },
                        children![
                            (
                                Text::new("Seed: "),
                                TextFont {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 12.0,
                                    ..default()
                                }
                            ),
                            (
                                Node {
                                    margin: UiRect::left(Val::Auto),
                                    ..Default::default()
                                },
                                Text::new(diagnostics.seed.to_string()),
                                TextFont {
                                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                                    font_size: 12.0,
                                    ..Default::default()
                                },
                                TextColor(palettes::css::GOLD.into()),
                                SeedText
                            )
                        ]
                    ),
                    (
                        Node {
                            width: Val::Percent(100.),
                            ..Default::default()
                        },
                        children![
                            (
                                Text::new("State: "),
                                TextFont {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 12.0,
                                    ..default()
                                }
                            ),
                            (
                                Node {
                                    margin: UiRect::left(Val::Auto),
                                    ..Default::default()
                                },
                                Text::default(),
                                TextFont {
                                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                                    font_size: 12.0,
                                    ..Default::default()
                                },
                                TextColor(palettes::css::GOLD.into()),
                                StateText
                            )
                        ]
                    ),
                ]
            ),
            (
                Node {
                    padding: UiRect::new(Val::Px(0.), Val::Px(0.), Val::Px(5.), Val::Px(5.)),
                    border: UiRect::bottom(Val::Px(1.)),
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                BorderColor(LinearRgba::new(0.2, 0.2, 0.2, 0.8).into()),
                children![
                    (
                        Node {
                            width: Val::Percent(100.),
                            display: Display::Flex,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..Default::default()
                        },
                        children![(
                            Text::new("Mesh generation"),
                            TextFont {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 14.0,
                                ..default()
                            }
                        ),]
                    ),
                    (
                        Node {
                            width: Val::Percent(100.),
                            ..Default::default()
                        },
                        children![
                            (
                                Text::new("Subdivisions: "),
                                TextFont {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 12.0,
                                    ..default()
                                }
                            ),
                            (
                                Node {
                                    margin: UiRect::left(Val::Auto),
                                    ..Default::default()
                                },
                                Text::default(),
                                TextFont {
                                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                                    font_size: 12.0,
                                    ..Default::default()
                                },
                                TextColor(palettes::css::GOLD.into()),
                                SubdivisionsText,
                            )
                        ]
                    ),
                    (
                        Node {
                            width: Val::Percent(100.),
                            ..Default::default()
                        },
                        children![
                            (
                                Text::new("Tiles: "),
                                TextFont {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 12.0,
                                    ..default()
                                }
                            ),
                            (
                                Node {
                                    margin: UiRect::left(Val::Auto),
                                    ..Default::default()
                                },
                                Text::default(),
                                TextFont {
                                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                                    font_size: 12.0,
                                    ..Default::default()
                                },
                                TextColor(palettes::css::GOLD.into()),
                                TileAmountText
                            )
                        ]
                    ),
                    (
                        Node {
                            width: Val::Percent(100.),
                            ..Default::default()
                        },
                        children![
                            (
                                Text::new("Time: "),
                                TextFont {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 12.0,
                                    ..default()
                                }
                            ),
                            (
                                Node {
                                    margin: UiRect::left(Val::Auto),
                                    ..Default::default()
                                },
                                Text::default(),
                                TextFont {
                                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                                    font_size: 12.0,
                                    ..Default::default()
                                },
                                TextColor(palettes::css::GOLD.into()),
                                MeshGenerationTimeText
                            )
                        ]
                    ),
                ]
            ),
            (
                Node {
                    padding: UiRect::new(Val::Px(0.), Val::Px(0.), Val::Px(5.), Val::Px(5.)),
                    border: UiRect::bottom(Val::Px(1.)),
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                BorderColor(LinearRgba::new(0.2, 0.2, 0.2, 0.8).into()),
                children![
                    (
                        Node {
                            width: Val::Percent(100.),
                            display: Display::Flex,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..Default::default()
                        },
                        children![(
                            Text::new("Tectonic simulation"),
                            TextFont {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 14.0,
                                ..default()
                            }
                        ),]
                    ),
                    (
                        Node {
                            width: Val::Percent(100.),
                            ..Default::default()
                        },
                        children![
                            (
                                Text::new("Particles: "),
                                TextFont {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 12.0,
                                    ..default()
                                }
                            ),
                            (
                                Node {
                                    margin: UiRect::left(Val::Auto),
                                    ..Default::default()
                                },
                                Text::default(),
                                TextFont {
                                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                                    font_size: 12.0,
                                    ..Default::default()
                                },
                                TextColor(palettes::css::GOLD.into()),
                                TectonicsParticleText
                            )
                        ]
                    ),
                    (
                        Node {
                            width: Val::Percent(100.),
                            ..Default::default()
                        },
                        children![
                            (
                                Text::new("Iteration: "),
                                TextFont {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 12.0,
                                    ..default()
                                }
                            ),
                            (
                                Node {
                                    margin: UiRect::left(Val::Auto),
                                    ..Default::default()
                                },
                                Text::default(),
                                TextFont {
                                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                                    font_size: 12.0,
                                    ..Default::default()
                                },
                                TextColor(palettes::css::GOLD.into()),
                                TectonicsIterationText
                            )
                        ]
                    ),
                    (
                        Node {
                            width: Val::Percent(100.),
                            ..Default::default()
                        },
                        children![
                            (
                                Text::new("Time: "),
                                TextFont {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 12.0,
                                    ..default()
                                }
                            ),
                            (
                                Node {
                                    margin: UiRect::left(Val::Auto),
                                    ..Default::default()
                                },
                                Text::default(),
                                TextFont {
                                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                                    font_size: 12.0,
                                    ..Default::default()
                                },
                                TextColor(palettes::css::GOLD.into()),
                                TectonicsTimeText
                            )
                        ]
                    )
                ]
            ),
            (
                Node {
                    padding: UiRect::new(Val::Px(0.), Val::Px(0.), Val::Px(5.), Val::Px(5.)),
                    border: UiRect::bottom(Val::Px(1.)),
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                BorderColor(LinearRgba::new(0.2, 0.2, 0.2, 0.8).into()),
                children![(
                    Node {
                        width: Val::Percent(100.),
                        display: Display::Flex,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    children![(
                        Text::new("Erosion simulation"),
                        TextFont {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 14.0,
                            ..default()
                        }
                    ),]
                ),]
            )
        ],
    ));
}
