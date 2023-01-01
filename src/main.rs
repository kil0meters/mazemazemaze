use bevy::{prelude::*, window::CursorGrabMode};
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_rapier3d::prelude::*;

use filter::FilterPlugin;
use marker::MarkerPlugin;
use maze2d::Maze2DPlugin;
use maze3d::Maze3DPlugin;
use menu::MenuPlugin;
use player2d::Player2DPlugin;
use player3d::Player3DPlugin;
use state::AppState;

mod direction;
mod filter;
mod marker;
mod maze2d;
mod maze3d;
mod menu;
mod player2d;
mod player3d;
mod state;

#[derive(Resource)]
pub struct Settings {
    sensitivity: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { sensitivity: 0.08 }
    }
}

fn main() {
    App::new()
        .init_resource::<Settings>()
        .add_state(AppState::MainMenu)
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(
            DefaultPlugins
                .build()
                .add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin),
        )
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(MenuPlugin)
        .add_plugin(Player2DPlugin)
        .add_plugin(Player3DPlugin)
        .add_plugin(Maze2DPlugin)
        .add_plugin(Maze3DPlugin)
        .add_plugin(FilterPlugin)
        .add_plugin(MarkerPlugin)
        .add_startup_system(play_music)
        .add_system(animate_spin)
        .add_system(cursor_grab)
        .run();
}

fn cursor_grab(
    keys: Res<Input<KeyCode>>,
    buttons: Res<Input<MouseButton>>,
    mut windows: ResMut<Windows>,
    state: Res<State<AppState>>,
) {
    if *state.current() != AppState::MainMenu {
        let window = windows.get_primary_mut().unwrap();

        if keys.just_pressed(KeyCode::Escape) {
            window.set_cursor_grab_mode(CursorGrabMode::None);
            window.set_cursor_visibility(true);
        }

        if buttons.just_pressed(MouseButton::Left) {
            window.set_cursor_grab_mode(CursorGrabMode::Locked);
            window.set_cursor_visibility(false);
        }
    }
}

#[derive(Component)]
pub struct Goal;

#[derive(Component)]
pub struct SpinBouncing;

fn animate_spin(mut query: Query<&mut Transform, With<SpinBouncing>>, time: Res<Time>) {
    for mut goal in query.iter_mut() {
        goal.rotate_y(time.delta_seconds());
        goal.translation.y += (time.elapsed_seconds() * 1.5).sin() * 0.005;
    }
}

fn play_music(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio.play_with_settings(
        asset_server.load("bg.ogg"),
        PlaybackSettings::LOOP.with_volume(1.0),
    );
}
