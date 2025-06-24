use bevy::{prelude::*, render::camera::ScalingMode, window::PrimaryWindow};
use bevy_asset_loader::prelude::*;
use bevy_tweening::{lens::*, *};


mod audio;
mod cursor;
pub mod level;
pub mod player;
pub mod ui;
pub mod utils;
pub mod menu;
pub mod selection; 
pub mod celebration;

use utils::*;

pub const RESIZE: f32 = 0.1;
pub const SPRITE_SIZE: f32 = 640.0 * RESIZE;

pub const MY_ORANGE: Color = Color::srgb(222.0 / 255.0, 112.0 / 255.0, 40.0 / 255.0);
pub const MY_BROWN: Color = Color::srgb(91.0 / 255.0, 75.0 / 255.0, 73.0 / 255.0);
pub const DARK_MODE_BG_COLOR: Color = Color::srgb(45.0 / 255.0, 47.0 / 255.0, 47.0 / 255.0);

pub const DUCK_MOVE_MILI_SECS: u64 = 300;

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameStates>()
            .add_loading_state(
                LoadingState::new(GameStates::Loading)
                    .continue_to_state(GameStates::GameMenu)
                    .load_collection::<AudioAssets>()
                    .load_collection::<ImageAssets>(),
            )
            .init_resource::<SelectedCharacters>()
            .add_plugins((
                player::Plugin,
                audio::Plugin,
                level::Plugin,
                ui::Plugin,
                cursor::Plugin,
                menu::Plugin,
                selection::Plugin,
                celebration::Plugin,
            ))
            .add_systems(Startup, spawn_camera);

    }
}

fn spawn_camera(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window = window_query.get_single().unwrap();
    let mut my_2d_camera_bundle = Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
        ..default()
    };
    my_2d_camera_bundle.projection.scaling_mode = ScalingMode::FixedHorizontal(1280.0);
    commands.spawn(my_2d_camera_bundle);
}

// TODO: How to scale all the ui elements?

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "audio/bgm.ogg")]
    bgm: Handle<AudioSource>,
    #[asset(path = "audio/eat.ogg")]
    eat: Handle<AudioSource>,
    #[asset(path = "audio/ice_breaking.ogg")]
    ice_breaking: Handle<AudioSource>,
    #[asset(path = "audio/quark.ogg")]
    quark: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(path = "sprites/arrow.png")]
    arrow: Handle<Image>,
    #[asset(path = "sprites/arrow2.png")]
    arrow2: Handle<Image>,
    #[asset(path = "sprites/bread.png")]
    bread: Handle<Image>,
    #[asset(path = "sprites/breaking_ice.png")]
    breaking_ice: Handle<Image>,
    #[asset(path = "sprites/duck.png")]
    duck: Handle<Image>,
    #[asset(path = "sprites/ice.png")]
    ice: Handle<Image>,
    #[asset(path = "sprites/stuffed_duck.png")]
    stuffed_duck: Handle<Image>,
    #[asset(path = "sprites/wall.png")]
    wall: Handle<Image>,
    #[asset(path = "sprites/water.png")]
    water: Handle<Image>,
    #[asset(path = "sprites/cat.png")]
    cat: Handle<Image>,
    #[asset(path = "sprites/stuffed_cat.png")]
    stuffed_cat: Handle<Image>,
    #[asset(path = "sprites/bunny.png")]
    bunny: Handle<Image>,
    #[asset(path = "sprites/stuffed_bunny.png")]
    stuffed_bunny: Handle<Image>,
    #[asset(path = "sprites/chick.png")]
    chick: Handle<Image>,
    #[asset(path = "sprites/stuffed_chick.png")]
    stuffed_chick: Handle<Image>,

}

// 添加角色类型
#[derive(Component, Default,Clone, Copy, PartialEq,serde::Deserialize,serde::Serialize, Eq, Debug)]
pub enum CharacterType {
    #[default]
    Duck,
    Cat,
    Chick,
    Bunny,
}

//player choose
#[derive(Resource, Default,serde::Serialize,serde::Deserialize)]

pub struct SelectedCharacters {
    pub player1: Option<CharacterType>,
    pub player2: Option<CharacterType>,
}
#[derive(Clone, Eq, Copy,PartialEq, Debug, Hash, Default, States,serde::Serialize,serde::Deserialize)]
pub enum GameStates {
    #[default]
    Loading,
    GameMenu,
    CharacterSelection,
    Next,
    Celebration,
}
