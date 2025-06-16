use bevy::color::palettes;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::text::ComputedTextBlock;

pub struct DebugUIPlugin;
impl Plugin for DebugUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, setup)
            .add_systems(Update, update_fps);
    }
}

#[derive(Component)]
struct FpsText;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Text with multiple sections
    commands.spawn((
        Node {
            width: Val::Px(200.),
            height: Val::Px(600.),
            margin: UiRect::with_left(UiRect::all(Val::Px(10.)), Val::Auto),
            padding: UiRect::all(Val::Px(10.)),
            display: Display::Grid,
            grid_template_columns: vec![GridTrack::min_content(), GridTrack::flex(1.0)],
            grid_template_rows: vec![GridTrack::auto()],
            ..Default::default()
        },
        BackgroundColor(LinearRgba::new(0.05, 0.05, 0.05, 0.8).into()),
        children![
            (
                Node {
                    display: Display::Grid,
                    ..Default::default()
                },
                Text::new("FPS: "),
                TextFont {
                    // This font is loaded and will be used instead of the default font.
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 18.0,
                    ..default()
                }
            ),
            (
                Node {
                    display: Display::Grid,
                    margin: UiRect::left(Val::Auto),
                    ..Default::default()
                },
                Text::default(),
                // "default_font" feature is unavailable, load a font to use instead.
                TextFont {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 18.0,
                    ..Default::default()
                },
                TextColor(palettes::css::GOLD.into()),
                FpsText
            )
        ],
    ));
}

fn update_fps(diagnostics: Res<DiagnosticsStore>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut span in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                **span = format!("{value:.0}");
            }
        }
    }
}
