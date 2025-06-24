// src/networking/client.rs
use bevy::prelude::*;
use bevy_renet::{
    renet::{
        transport::{ClientAuthentication, NetcodeClientTransport},
         RenetClient,
    },
    RenetClientPlugin,
};
use std::{net::UdpSocket, time::SystemTime};
use renet::ConnectionConfig;
use crate::{game::GameStates, networking::ServerAddress};
use crate::game::player::{Player1,Player2};
use super::{
    ClientChannels, ClientMessage, FullGameState, ServerMessage, PROTOCOL_ID,
};
use crate::game::player::CommonDuck;
use crate::game::level::{CurrentLevelIndex,Level};
use crate::game::utils::Direction;
pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RenetClientPlugin)
            .add_plugins(bevy_renet::transport::NetcodeClientPlugin)
            .init_resource::<ClientChannels>()
            .init_resource::<FullGameState>()
            .init_resource::<SelectionState>()
            .add_event::<RemotePlayerMove>()
            .add_systems(OnEnter(GameStates::Loading), setup_client)
            .add_systems(
                Update,
                (
                    handle_server_messages,
                    send_character_selection.run_if(in_state(GameStates::CharacterSelection)),
                    send_player_movement.run_if(in_state(GameStates::Next)),
                    send_level_control_requests.run_if(in_state(GameStates::Next)),
                    send_change_level_request.run_if(in_state(GameStates::Next))
                ),
            );
    }
}

fn setup_client(mut commands: Commands, server_ip: Res<ServerAddress>) {
    let server_addr = format!("{}:5000",server_ip.0).parse().unwrap();
    let client_id = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let connection_config = ConnectionConfig::default();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let client = RenetClient::new(connection_config);
    let transport = NetcodeClientTransport::new(
        current_time,
        ClientAuthentication::Unsecure {
            client_id,
            protocol_id: PROTOCOL_ID,
            server_addr,
            user_data: None,
        },
        socket,
    )
    .unwrap();

    commands.insert_resource(client);
    commands.insert_resource(transport);

    info!("Client started, connecting to {}", server_addr);
}

use crate::game::SelectedCharacters;
use crate::networking::SelectionState;
use crate::networking::RemotePlayerMove;
use crate::game::level::{RestartLevelEvent,UndoLevelEvent,ChangeLevelEvent};
fn handle_server_messages(
    mut client: ResMut<RenetClient>,
    mut game_state: ResMut<FullGameState>,
    mut next_state: ResMut<NextState<GameStates>>,
    client_channels: Res<ClientChannels>,
    mut player1_query: Query<(&mut Transform,&mut CommonDuck), (With<Player1>,Without<Player2>)>,
    mut player2_query: Query<(&mut Transform,&mut CommonDuck) ,(With<Player2>,Without<Player1>)>,
    current_state: Res<State<GameStates>>, // 添加当前状态查询
    mut selected_characters: ResMut<SelectedCharacters>,
    mut selection_state: ResMut<SelectionState>,
    //mut commands: Commands,
    mut event_writer: EventWriter<RemotePlayerMove>,
    mut current_level_index: ResMut<CurrentLevelIndex>,

    mut restart_writer: EventWriter<RestartLevelEvent>,
    mut undo_writer: EventWriter<UndoLevelEvent>,
    mut change_writer: EventWriter<ChangeLevelEvent>,


) {
    // 处理可靠消息
    while let Some(message) = client.receive_message(client_channels.reliable_ordered) {
        if let Ok(server_message) = bincode::deserialize::<ServerMessage>(&message) {
            match server_message {

                ServerMessage::StateChangeNotification(new_state) => {
                    // 比较当前状态和新状态是否不同
                    if *current_state.get() != new_state {
                        next_state.set(new_state);
                        game_state.current_state = new_state;
                        info!("Received state change notification: {:?}", new_state);
                    }
                }
                ServerMessage::FullStateSync(full_state) => {
                    *game_state = full_state.clone();

                    if let Ok((mut transform, mut duck)) = player1_query.get_single_mut() {
                        transform.translation = full_state.player1_position;
                        duck.logic_position = full_state.player1_logic_pos;
                        duck.bread_count = full_state.player1_bread;
                        duck.can_move = full_state.player1_can_move;
                    }

                    if let Ok((mut transform, mut duck)) = player2_query.get_single_mut() {
                        transform.translation = full_state.player2_position;
                        duck.logic_position = full_state.player2_logic_pos;
                        duck.bread_count = full_state.player2_bread;
                        duck.can_move = full_state.player2_can_move;
                    }

                    if *current_state.get() != game_state.current_state {
                        next_state.set(game_state.current_state);
                    }
                }

                ServerMessage::PlayerPositionUpdate { player_id, position } => {
                    if player_id == 1 {
                        game_state.player1_position = position;
                        if let Ok((mut transform, _)) = player1_query.get_single_mut() {
                            transform.translation = position;
                        }
                    } else if player_id == 2 {
                        game_state.player2_position = position;
                        if let Ok((mut transform, _)) = player2_query.get_single_mut() {
                            transform.translation = position;
                        }
                    }
                }

                ServerMessage::CharacterSelectionUpdate { player1_choice, player2_choice } => {
                    selection_state.player1_choice = player1_choice;
                    selection_state.player2_choice = player2_choice;
                    info!("Received character selection update: P1={:?}, P2={:?}", 
                        player1_choice, player2_choice);
                },
                ServerMessage::StartGameWithCharacters { 
                    player1_character, 
                    player2_character 
                } => {
                    // 更新选择的角色
                    selected_characters.player1 = Some(player1_character);
                    selected_characters.player2 = Some(player2_character);
                    game_state.player1_character = player1_character;
                    game_state.player2_character = player2_character;
                    
                    // 切换到游戏状态
                    next_state.set(GameStates::Next);
                    info!("Starting game with characters: P1={:?}, P2={:?}",
                        player1_character, player2_character);
                },

                //人物移动
                ServerMessage::PlayerMovementUpdate {
                    player_id,
                    direction
                } => {
                    event_writer.send(RemotePlayerMove { player_id, direction });
                }


                ServerMessage::NextLevelNotification{level_index} => {
                    info!("currentlevel :{}",current_level_index.0);
                    current_level_index.0 = level_index.0;
                    next_state.set(GameStates::Next);
                    info!("levelnotification:{}",level_index.0);
                }

                ServerMessage::DoRestartLevel => {
                    restart_writer.send(RestartLevelEvent);
                }

                ServerMessage::DoUndoLevel(new_level) => {
                    undo_writer.send(UndoLevelEvent { level_data: new_level });
                }

                ServerMessage::DoChangeLevel(index) => {
                    change_writer.send(ChangeLevelEvent { index });
                }

            }
        }
    }
}

fn send_character_selection(
    mut client: Option<ResMut<RenetClient>>,
    selected_characters: ResMut<SelectedCharacters>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if let Some(client) = &mut client {
        // Send character selection when Enter is pressed
        if keyboard.just_pressed(KeyCode::Enter) {
            info!("selected_characters.player1: {:?}", selected_characters.player1);
            if let Some(character) = selected_characters.player1 {
                info!("2");
                let message = bincode::serialize(&ClientMessage::CharacterSelected {
                    player_id: 1,
                    character,
                }).unwrap();
                client.send_message(0, message);
                info!("Player1 character selection sent to server: {:?}", character);
            }
            
            info!("selected_characters.player2: {:?}", selected_characters.player2);
            if let Some(character) = selected_characters.player2 {
                info!("2");
                let message = bincode::serialize(&ClientMessage::CharacterSelected {
                    player_id: 2,
                    character,
                }).unwrap();
                client.send_message(0, message);
                info!("Player2 character selection sent to server: {:?}", character);
            }
            
            // Send ready notification
            let message = bincode::serialize(&ClientMessage::ReadyForGameStart).unwrap();
            client.send_message(0, message);
            info!("ReadyForGameStart message sent");
        }
    }
}

fn send_player_movement(
    mut client: Option<ResMut<RenetClient>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if let Some(client) = &mut client {
        let mut send_move = |player_id: u8, direction: Direction| {
            let msg = ClientMessage::PlayerMovementInput { player_id, direction } ;
            let serialized = bincode::serialize(&msg).unwrap();
            client.send_message(0, serialized);
        };

        // player1 - WASD
        if keyboard.just_pressed(KeyCode::KeyW) {
            send_move(1, Direction::Up);
        }
        if keyboard.just_pressed(KeyCode::KeyS) {
            send_move(1, Direction::Down);
        }
        if keyboard.just_pressed(KeyCode::KeyA) {
            send_move(1, Direction::Left);
        }
        if keyboard.just_pressed(KeyCode::KeyD) {
            send_move(1, Direction::Right);
        }

        // player2 - arrow keys
        if keyboard.just_pressed(KeyCode::ArrowUp) {
            send_move(2, Direction::Up);
        }
        if keyboard.just_pressed(KeyCode::ArrowDown) {
            send_move(2, Direction::Down);
        }
        if keyboard.just_pressed(KeyCode::ArrowLeft) {
            send_move(2, Direction::Left);
        }
        if keyboard.just_pressed(KeyCode::ArrowRight) {
            send_move(2, Direction::Right);
        }
    }
}

fn send_level_control_requests(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut client: Option<ResMut<RenetClient>>,
) {
    if let Some(client) = &mut client {
        if keyboard.just_pressed(KeyCode::KeyR) {
            let msg = bincode::serialize(&ClientMessage::RestartLevel).unwrap();
            client.send_message(0, msg);
        }

        if keyboard.just_pressed(KeyCode::KeyZ) {
            info!("sent undo");
            let msg = bincode::serialize(&ClientMessage::UndoLevel).unwrap();
            client.send_message(0, msg);
        }
    }
}

use crate::game::level::Levels;
use crate::game::level::load_level;
fn send_change_level_request(
    input: Res<ButtonInput<KeyCode>>,
    levels: Res<Levels>,
    mut level_index: ResMut<CurrentLevelIndex>,
    mut client: Option<ResMut<RenetClient>>,
) {
    let mut new_index = level_index.0;

    if input.just_pressed(KeyCode::BracketLeft) && new_index > 1 {
        new_index -= 1;
    }
    if input.just_pressed(KeyCode::BracketRight) {
        new_index += 1;
    }

    if new_index !=level_index.0{
        if load_level(new_index, levels).is_ok() {
            
            level_index.0 = new_index;

            if let Some(client) = &mut client {
                let msg = ClientMessage::ChangeLevelCheat(CurrentLevelIndex(new_index));
                let serialized = bincode::serialize(&msg).unwrap();
                client.send_message(0, serialized);
            }
        }
    }

}
