#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_wasm_window_resize::WindowResizePlugin;
use bevy_tweening::TweeningPlugin;

mod game;
mod networking;
use networking::ServerAddress;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let is_server = args.iter().any(|arg| arg == "--server");

    // 仅用于显示窗口标题和日志
    let client_identity = if args.iter().any(|arg| arg == "--player1") {
        Some("Player1")
    } else if args.iter().any(|arg| arg == "--player2") {
        Some("Player2")
    } else {
        None
    };

    let server_ip = args.iter()
    .position(|arg| arg == "--connect")
    .and_then(|i| args.get(i + 1))
    .cloned()
    .unwrap_or("127.0.0.1".to_string());


    let window_title = if is_server {
        "Bevy Jam 4 🦀 -server".into()
    } else if let Some(id) = client_identity {
        format!("Bevy Jam 4 🦀 -{}", id)
    } else {
        "Bevy Jam 4 🦀".into()
    };

    let mut app = App::new();

    app.insert_resource(ClearColor(game::DARK_MODE_BG_COLOR))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: window_title,
                        mode: bevy::window::WindowMode::Windowed,
                        prevent_default_event_handling: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..Default::default()
                }),
        )
        .add_plugins(TweeningPlugin)
        .add_plugins(WindowResizePlugin)
        .add_plugins(game::Plugin)
        .init_state::<game::GameStates>();

    if is_server {
        app.add_plugins(networking::server::ServerPlugin);
        info!("Running as SERVER.");
    } else {
        app.insert_resource(ServerAddress(server_ip));
        app.add_plugins(networking::client::ClientPlugin);
        if let Some(id) = client_identity {
            info!("Running as CLIENT with identity hint: {}", id);
        } else {
            info!("Running as CLIENT, waiting for identity assignment.");
        }
    }

    app.run();
}
