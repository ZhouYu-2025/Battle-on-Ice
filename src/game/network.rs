use bevy::prelude::*;
use bevy_renet::renet::{
    RenetClient, RenetServer, ServerEvent, DefaultChannel, ConnectionConfig
};
use bevy_renet::renet::transport::{
    NetcodeClientTransport, NetcodeServerTransport, ClientAuthentication,ServerConfig
};
use bevy_renet::{RenetClientPlugin, RenetServerPlugin};
use serde::{Serialize, Deserialize};
use std::net::{SocketAddr, UdpSocket};
use std::time::SystemTime;

use crate::game::{
    GameStates, CharacterType, SelectedCharacters, CurrentLevelIndex,
    player::{Player1, Player2, CommonDuck},
    level::Level,
    utils::Direction
};

const PROTOCOL_ID: u64 = 7; // 选择一个唯一的协议ID

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum NetworkMessage {
    // 游戏状态同步
    GameState(GameStates),
    // 关卡数据
    LevelData {
        index: usize,
        data: Vec<Vec<char>>,
    },
    // 玩家选择
    PlayerSelection {
        player: PlayerType,
        character: CharacterType,
    },
    // 玩家移动
    PlayerMove {
        player: PlayerType,
        direction: Direction,
        start_pos: (usize, usize),
        end_pos: (usize, usize),
    },
    // 玩家位置同步
    PlayerPosition {
        player: PlayerType,
        position: (usize, usize),
        bread_count: u32,
    },
    // 游戏事件
    GameEvent(GameEvent),
    // Ping测试
    Ping(u64),
    // Pong响应
    Pong(u64),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GameEvent {
    LevelComplete,
    PlayerStuck,
    BreadEaten,
    IceBroken,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum PlayerType {
    Player1,
    Player2,
}

impl PlayerType {
    pub fn is_player1(&self) -> bool {
        matches!(self, PlayerType::Player1)
    }
}

#[derive(Resource, Default)]
pub struct NetworkResource {
    pub player_type: Option<PlayerType>,
    pub connected: bool,
    pub ping: u64,
}

#[derive(Resource)]
pub struct ServerLobby {
    pub players: Vec<PlayerType>,
}

impl Default for ServerLobby {
    fn default() -> Self {
        Self { players: Vec::with_capacity(2) }
    }
}
// ==== Configuration struct ====

#[derive(Resource)]
pub struct NetworkConfig {
    pub is_server: bool,
}

impl NetworkConfig {
    pub fn from_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let is_server = args.contains(&"--server".to_string());
        NetworkConfig { is_server }
    }
}

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetworkResource>()
           .init_resource::<ServerLobby>()
           .add_plugins((
               RenetClientPlugin,
               RenetServerPlugin,

           ));
        
        //运行时根据参数决定注册哪些系统
        if app.world.resource::<NetworkConfig>().is_server {
            app.add_systems(Startup, setup_server)
               .add_systems(Update, (
                   server_event_system,
                   server_receive_messages,
                   broadcast_game_state,
               ));
        } else {
            app.add_systems(Startup, setup_client)
               .add_systems(Update, (
                   client_send_input,
                   client_receive_messages,
                   update_ping,
               ));
        }
    }
}

// 服务器设置
fn setup_server(mut commands: Commands) {
    let server_addr: SocketAddr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind(server_addr).unwrap();
    
    let connection_config = ConnectionConfig::default();
    let server = RenetServer::new(connection_config);
    
    let server_config = ServerConfig {
        max_clients: 2,
        protocol_id: PROTOCOL_ID,
        public_addr: server_addr,
        authentication: ServerAuthentication::Unsecure,
    };
    
    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();
    
    commands.insert_resource(server);
    commands.insert_resource(transport);
    
    info!("Server started on {}", server_addr);
}

// 客户端设置
fn setup_client(mut commands: Commands) {
    let client = RenetClient::new(ConnectionConfig::default());
    
    let server_addr: SocketAddr = "127.0.0.1:5000".parse().unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };
    
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();
    
    commands.insert_resource(client);
    commands.insert_resource(transport);
    commands.insert_resource(NetworkResource::default());
    
    info!("Client connected to server");
}

// 服务器事件处理
fn server_event_system(
    mut server_events: EventReader<ServerEvent>,
    mut lobby: ResMut<ServerLobby>,
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                info!("Client {} connected", client_id);
                
                // 分配玩家角色
                let player_type = if !lobby.players.contains(&PlayerType::Player1) {
                    PlayerType::Player1
                } else if !lobby.players.contains(&PlayerType::Player2) {
                    PlayerType::Player2
                } else {
                    // 已经有两个玩家了
                    server.disconnect(*client_id);
                    continue;
                };
                
                lobby.players.push(player_type);
                
                // 通知客户端他们的玩家类型
                let message = NetworkMessage::PlayerSelection {
                    player: player_type,
                    character: CharacterType::Duck, // 默认角色
                };
                
                server.send_message(
                    *client_id,
                    DefaultChannel::ReliableOrdered,
                    bincode::serialize(&message).unwrap(),
                );
                
                // 如果两个玩家都连接了，开始游戏
                if lobby.players.len() == 2 {
                    let message = NetworkMessage::GameState(GameStates::CharacterSelection);
                    server.broadcast_message(
                        DefaultChannel::ReliableOrdered,
                        bincode::serialize(&message).unwrap(),
                    );
                }
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("Client {} disconnected: {}", client_id, reason);
                
                // 从大厅移除玩家
                if let Some(index) = lobby.players.iter().position(|&p| {
                    p as u64 == *client_id
                }) {
                    lobby.players.remove(index);
                }
                
                // 通知另一个玩家游戏结束
                let message = NetworkMessage::GameState(GameStates::GameMenu);
                server.broadcast_message(
                    DefaultChannel::ReliableOrdered,
                    bincode::serialize(&message).unwrap(),
                );
            }
        }
    }
}

// 客户端发送输入
pub fn client_send_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut client: ResMut<RenetClient>,
    network: Res<NetworkResource>,
    selected_characters: Res<SelectedCharacters>,
    duck_query: Query<(&CommonDuck, Option<&Player1>, Option<&Player2>)>,
) {
    // 只允许当前玩家发送输入
    let Some(player_type) = network.player_type else { return };
    
    // 发送玩家选择
    if let Some(character) = selected_characters.player1 {
        if player_type == PlayerType::Player1 {
            let message = NetworkMessage::PlayerSelection {
                player: PlayerType::Player1,
                character,
            };
            client.send_message(
                DefaultChannel::ReliableOrdered,
                bincode::serialize(&message).unwrap(),
            );
        }
    }
    
    if let Some(character) = selected_characters.player2 {
        if player_type == PlayerType::Player2 {
            let message = NetworkMessage::PlayerSelection {
                player: PlayerType::Player2,
                character,
            };
            client.send_message(
                DefaultChannel::ReliableOrdered,
                bincode::serialize(&message).unwrap(),
            );
        }
    }
    
    // 发送移动输入
    for (duck, p1, p2) in duck_query.iter() {
        let is_current_player = match (p1, p2, player_type) {
            (Some(_), None, PlayerType::Player1) => true,
            (None, Some(_), PlayerType::Player2) => true,
            _ => false,
        };
        
        if !is_current_player {
            continue;
        }
        
        let direction = if keyboard_input.just_pressed(KeyCode::KeyW) {
            Some(Direction::Up)
        } else if keyboard_input.just_pressed(KeyCode::KeyS) {
            Some(Direction::Down)
        } else if keyboard_input.just_pressed(KeyCode::KeyA) {
            Some(Direction::Left)
        } else if keyboard_input.just_pressed(KeyCode::KeyD) {
            Some(Direction::Right)
        } else {
            None
        };
        
        if let Some(direction) = direction {
            let message = NetworkMessage::PlayerMove {
                player: player_type,
                direction,
                start_pos: duck.logic_position,
                end_pos: duck.logic_position, // 服务器会计算实际位置
            };
            
            client.send_message(
                DefaultChannel::ReliableOrdered,
                bincode::serialize(&message).unwrap(),
            );
        }
    }
    
    // 定期发送ping
    if SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() % 2 == 0
    {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        client.send_message(
            DefaultChannel::Unreliable,
            bincode::serialize(&NetworkMessage::Ping(timestamp)).unwrap(),
        );
    }
}

// 客户端接收消息
fn client_receive_messages(
    mut client: ResMut<RenetClient>,
    mut network: ResMut<NetworkResource>,
    mut game_state: ResMut<NextState<GameStates>>,
    mut level_index: ResMut<CurrentLevelIndex>,
    mut level: ResMut<Level>,
    mut selected_characters: ResMut<SelectedCharacters>,
    mut duck_query: Query<(&mut CommonDuck, Option<&Player1>, Option<&Player2>)>,
) {
    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        if let Ok(network_message) = bincode::deserialize::<NetworkMessage>(&message) {
            match network_message {
                NetworkMessage::GameState(state) => {
                    game_state.set(state);
                }
                NetworkMessage::LevelData { index, data } => {
                    level_index.0 = index;
                    level.0 = data;
                }
                NetworkMessage::PlayerSelection { player, character } => {
                    network.player_type = Some(player);
                    match player {
                        PlayerType::Player1 => selected_characters.player1 = Some(character),
                        PlayerType::Player2 => selected_characters.player2 = Some(character),
                    }
                }
                NetworkMessage::PlayerPosition { player, position, bread_count } => {
                    for (mut duck, p1, p2) in duck_query.iter_mut() {
                        let is_player = match (p1, p2, player) {
                            (Some(_), None, PlayerType::Player1) => true,
                            (None, Some(_), PlayerType::Player2) => true,
                            _ => false,
                        };
                        
                        if is_player {
                            duck.logic_position = position;
                            duck.bread_count = bread_count;
                        }
                    }
                }
                NetworkMessage::Pong(timestamp) => {
                    let now = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;
                    network.ping = now - timestamp;
                }
                _ => {}
            }
        }
    }
}

// 服务器接收消息
fn server_receive_messages(
    mut server: ResMut<RenetServer>,
    mut level_index: ResMut<CurrentLevelIndex>,
    mut level: ResMut<Level>,
    mut game_state: ResMut<NextState<GameStates>>,
    lobby: Res<ServerLobby>,
    mut duck_query: Query<(&mut CommonDuck, Option<&Player1>, Option<&Player2>)>,
) {
    for client_id in server.clients_id().into_iter() {
        while let Some(message) = server.receive_message(client_id, DefaultChannel::ReliableOrdered) {
            if let Ok(network_message) = bincode::deserialize::<NetworkMessage>(&message) {
                match network_message {
                    NetworkMessage::PlayerMove { player, direction, start_pos, .. } => {
                        // 验证移动
                        for (mut duck, p1, p2) in duck_query.iter_mut() {
                            let is_player = match (p1, p2, player) {
                                (Some(_), None, PlayerType::Player1) => true,
                                (None, Some(_), PlayerType::Player2) => true,
                                _ => false,
                            };
                            
                            if is_player && duck.logic_position == start_pos && duck.can_move {
                                // 简单验证后广播移动结果
                                let end_pos = calculate_move(duck.logic_position, direction, &level);
                                
                                let message = NetworkMessage::PlayerPosition {
                                    player,
                                    position: end_pos,
                                    bread_count: duck.bread_count,
                                };
                                
                                server.broadcast_message(
                                    DefaultChannel::ReliableOrdered,
                                    bincode::serialize(&message).unwrap(),
                                );
                            }
                        }
                    }
                    NetworkMessage::Ping(timestamp) => {
                        // 返回pong
                        server.send_message(
                            client_id,
                            DefaultChannel::Unreliable,
                            bincode::serialize(&NetworkMessage::Pong(timestamp)).unwrap(),
                        );
                    }
                    _ => {}
                }
            }
        }
    }
}

// 广播游戏状态
fn broadcast_game_state(
    level: Res<Level>,
    level_index: Res<CurrentLevelIndex>,
    game_state: Res<NextState<GameStates>>,
    server: Option<ResMut<RenetServer>>,
    duck_query: Query<(&CommonDuck, Option<&Player1>, Option<&Player2>)>,
) {
    if let Some(mut server) = server {
        // 如果关卡变化，发送整个关卡数据
        if level.is_changed() {
            let message = NetworkMessage::LevelData {
                index: level_index.0,
                data: level.0.clone(),
            };
            
            server.broadcast_message(
                DefaultChannel::ReliableOrdered,
                bincode::serialize(&message).unwrap(),
            );
        }
        
        // 定期发送玩家位置
        for (duck, p1, p2) in duck_query.iter() {
            let player_type = if p1.is_some() {
                PlayerType::Player1
            } else if p2.is_some() {
                PlayerType::Player2
            } else {
                continue;
            };
            
            let message = NetworkMessage::PlayerPosition {
                player: player_type,
                position: duck.logic_position,
                bread_count: duck.bread_count,
            };
            
            server.broadcast_message(
                DefaultChannel::Unreliable,
                bincode::serialize(&message).unwrap(),
            );
        }
    }
}

// 更新ping值
fn update_ping(
    mut network: ResMut<NetworkResource>,
    client: Option<ResMut<RenetClient>>,
) {
    if let Some(mut client) = client {
        while let Some(message) = client.receive_message(DefaultChannel::Unreliable) {
            if let Ok(NetworkMessage::Pong(timestamp)) = bincode::deserialize(&message) {
                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
                network.ping = now - timestamp;
            }
        }
    }
}

// 计算移动位置 (简化版)
fn calculate_move(
    start_pos: (usize, usize),
    direction: Direction,
    level: &Level,
) -> (usize, usize) {
    // 这里应该实现与游戏逻辑相同的移动计算
    // 简化版只做基本移动
    match direction {
        Direction::Up => (start_pos.0.saturating_sub(1), start_pos.1),
        Direction::Down => (start_pos.0 + 1, start_pos.1),
        Direction::Left => (start_pos.0, start_pos.1.saturating_sub(1)),
        Direction::Right => (start_pos.0, start_pos.1 + 1),
        Direction::None => start_pos,
    }
}