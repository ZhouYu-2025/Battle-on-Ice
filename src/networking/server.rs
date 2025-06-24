// src/networking/server.rs
use bevy::prelude::*;
use bevy_renet::{
    renet::{RenetServer, transport::{NetcodeServerTransport, ServerConfig, ServerAuthentication}},
    RenetServerPlugin,
};
use std::{net::UdpSocket};
use crate::{networking::SelectionState};
use crate::game::level::{Level,Levels};
use super::*;
use renet::ServerEvent;
use std::time::SystemTime;
use renet::ConnectionConfig;
pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RenetServerPlugin)
            .add_plugins(bevy_renet::transport::NetcodeServerPlugin)
            .init_resource::<ServerChannels>()
            .init_resource::<SelectionState>()
            .add_systems(OnEnter(GameStates::Loading), setup_server)
            .add_systems(Update, (handle_client_messages, handle_server_events));
    }
}

fn setup_server(mut commands: Commands) {
    let server_addr = "0.0.0.0:5000".parse().unwrap();
    let socket = UdpSocket::bind(server_addr).unwrap();
    let connection_config = ConnectionConfig::default();
    let server_config = ServerConfig {
        current_time: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap(),
        max_clients: 2,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![server_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();
    let server = RenetServer::new(connection_config);

    commands.insert_resource(server);
    commands.insert_resource(transport);
    commands.insert_resource(FullGameState::default());

    info!("Server started on {:?}", server_addr);
}

use crate::game::level::LevelStack;
use crate::game::level::BreadSumRecordStack;
fn handle_client_messages(
    mut server: ResMut<RenetServer>,
    mut selection_state: ResMut<SelectionState>,
    mut game_state: ResMut<FullGameState>,
    server_channels: Res<ServerChannels>,
    mut next_state: ResMut<NextState<GameStates>>,
    levels: Res<Levels>,
    mut level: ResMut<Level>,
    mut level_stack: ResMut<LevelStack>,
    mut bread_sum_record_stack: ResMut<BreadSumRecordStack>,
    /* 
    mut bread_count: ResMut<BreadCount>,
    mut level_index: ResMut<CurrentLevelIndex>,
    mut events_update: EventWriter<UpdateLevel>,*/
) {
    for client_id in server.clients_id().into_iter() {
        //info!("start handle message");
        // 处理可靠消息
        while let Some(message) = server.receive_message(client_id, server_channels.reliable_ordered) {
            info!("receive message from:{}",client_id);
            if let Ok(client_message) = bincode::deserialize::<ClientMessage>(&message) {
                match client_message {
                    ClientMessage::StateChangeRequest(new_state) => {
                        game_state.current_state = new_state;
                        next_state.set(new_state);
                        
                        // 广播新状态给所有客户端
                        let message = bincode::serialize(&ServerMessage::StateChangeNotification(
                            new_state,
                        ))
                        .unwrap();
                        server.broadcast_message(server_channels.reliable_ordered, message);
                        
                        info!("State changed to: {:?}", new_state);
                        
                    }
                    ClientMessage::PlayerPositionUpdate(position) => {
                        // 更新玩家位置并广播
                        let player_id = if server.clients_id()[0] == client_id { 1 } else { 2 };
                        
                        if player_id == 1 {
                            game_state.player1_position = position;
                        } else {
                            game_state.player2_position = position;
                        }
                        
                        let message = bincode::serialize(&ServerMessage::PlayerPositionUpdate {
                            player_id,
                            position,
                        })
                        .unwrap();
                        server.broadcast_message(server_channels.unreliable, message);
                    }
                    
                    ClientMessage::RequestFullState => {
                        // 发送完整状态给请求的客户端
                        let message = bincode::serialize(&ServerMessage::FullStateSync(
                            game_state.clone(),
                        ))
                        .unwrap();
                        server.send_message(client_id, server_channels.reliable_ordered, message);
                    }
                    ClientMessage::CharacterSelected { player_id, character } => {
                        match player_id {
                            1 => {
                                selection_state.player1_choice = Some(character);
                                info!("Player 1 selected {:?}", character);
                            },
                            2 => {
                                selection_state.player2_choice = Some(character);
                                info!("Player 2 selected {:?}", character);
                            },
                            _ => {}
                        }
                        
                        // 广播选择更新给所有客户端
                        let update = ServerMessage::CharacterSelectionUpdate {
                            player1_choice: selection_state.player1_choice,
                            player2_choice: selection_state.player2_choice,
                        };
                        let message = bincode::serialize(&update).unwrap();
                        server.broadcast_message(0, message);
                    },

                    ClientMessage::ReadyForGameStart => {
                        let player_id = if server.clients_id()[0] == client_id { 1 } else { 2 };
                        match player_id {
                            1 => {
                                selection_state.player1_ready = true;
                                info!("Player 1 is ready");
                            },
                            2 => {
                                selection_state.player2_ready = true;
                                info!("Player 2 is ready");
                            },
                            _ => {}
                        }
                        
                        // 检查是否都准备好了
                        if selection_state.player1_ready && selection_state.player2_ready {
                            info!("Both players ready. p1_choice: {:?}, p2_choice: {:?}", selection_state.player1_choice, selection_state.player2_choice);    
                            if let (Some(p1_char), Some(p2_char)) = (
                                selection_state.player1_choice,
                                selection_state.player2_choice
                            ) {
                                // 通知所有客户端开始游戏
                                let start_msg = ServerMessage::StartGameWithCharacters {
                                    player1_character: p1_char,
                                    player2_character: p2_char,
                                };
                                info!("start!");
                                let message = bincode::serialize(&start_msg).unwrap();
                                server.broadcast_message(0, message);
                                
                                // 更新服务器状态
                                game_state.player1_character = p1_char;
                                game_state.player2_character = p2_char;
                                next_state.set(GameStates::Next);
                            }
                        }
                    },

                    //movement
                    ClientMessage::PlayerMovementInput { player_id, direction } => {
                        let msg = ServerMessage::PlayerMovementUpdate { player_id, direction };
                        let serialized = bincode::serialize(&msg).unwrap();
                        server.broadcast_message(server_channels.reliable_ordered, serialized);
                    }

                    ClientMessage::NextLevelRequest => {
                        game_state.current_level.0 += 1;

                        let new_level = game_state.current_level;

                        info!("Server: advancing to level {}", new_level.0);

                        let message = bincode::serialize(&ServerMessage::NextLevelNotification {
                            level_index: new_level,
                        }).unwrap();

                        server.broadcast_message(server_channels.reliable_ordered, message);
                        
                    }

                    ClientMessage::RestartLevel => {
                        let msg = ServerMessage::DoRestartLevel;
                        let serialized = bincode::serialize(&msg).unwrap();
                        server.broadcast_message(server_channels.reliable_ordered, serialized);
                    }

                    ClientMessage::UndoLevel => {
                        info!("Undo request from client {}", client_id);

                        // 至少有两帧状态才能撤销（当前和之前）
                        if level_stack.0.size() >= 2 {
                            // 撤销 Level 和 Bread 状态
                            level_stack.0.pop();
                            let popped_level = level_stack.0.peek().unwrap().clone();
                            level.0 = popped_level.clone();

                            if bread_sum_record_stack.0.size() >= 1 {
                                bread_sum_record_stack.0.pop();
                            }

                            let msg = bincode::serialize(&ServerMessage::DoUndoLevel(Level(popped_level))).unwrap();
                            server.broadcast_message(server_channels.reliable_ordered, msg);
                            info!("Server broadcasted undo level");
                        } else {
                            info!("Undo rejected: level stack too shallow");
                        }
                    }
                    

                    ClientMessage::ChangeLevelCheat(index) => {

                        let msg = ServerMessage::DoChangeLevel(index);
                        let serialized = bincode::serialize(&msg).unwrap();
                        server.broadcast_message(server_channels.reliable_ordered, serialized);
                        info!("change to{}",index.0);
                    }


                }
            }
        }
    }
}

fn handle_server_events(
    mut server: ResMut<RenetServer>,
    mut server_events: EventReader<ServerEvent>,
    server_channels: Res<ServerChannels>,
    game_state: ResMut<FullGameState>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                info!("Client {} connected", client_id);
                
                // 分配玩家ID
                let player_id = if server.clients_id().len() == 1 { 1 } else { 2 };
                info!("Assigned Player {} to client {}", player_id, client_id);
                
                // 发送完整状态给新客户端
                let message = bincode::serialize(&ServerMessage::FullStateSync(
                    game_state.clone(),
                ))
                .unwrap();
                server.send_message(*client_id, server_channels.reliable_ordered, message);
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("Client {} disconnected: {}", client_id, reason);
                // 可以在这里处理玩家断开后的逻辑
            }
        }
    }
}
