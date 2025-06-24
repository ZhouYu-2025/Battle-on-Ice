// src/networking/mod.rs
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::game::level::CurrentLevelIndex;
use crate::game::GameStates;
// 协议ID，用于客户端和服务器之间匹配
pub const PROTOCOL_ID: u64 = 7;
use crate::game::utils::Direction;
use crate::game::level::Level;

use bevy::prelude::Resource;
//server address
#[derive(Resource, Debug, Clone)]
pub struct ServerAddress(pub String);

/// 定义网络通道
#[derive(Debug, Resource)]
pub struct ClientChannels {
    pub reliable_ordered: u8, // 可靠有序通道，用于状态同步
    pub unreliable: u8,       // 不可靠无序通道，用于玩家位置更新
}

impl Default for ClientChannels {
    fn default() -> Self {
        ClientChannels {
            reliable_ordered: 0,
            unreliable: 1,
        }
    }
}

#[derive(Debug, Resource)]
pub struct ServerChannels {
    pub reliable_ordered: u8,
    pub unreliable: u8,
}

impl Default for ServerChannels {
    fn default() -> Self {
        ServerChannels {
            reliable_ordered: 0,
            unreliable: 1,
        }
    }
}

use crate::game::CharacterType;
/// 客户端发送给服务器的消息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ClientMessage {
    /// 游戏状态改变请求
    StateChangeRequest(GameStates),
    /// 玩家位置更新
    PlayerPositionUpdate(Vec3),
    /// 请求完整游戏状态
    RequestFullState,
    //人物选择
    CharacterSelected {
        player_id: u8,
        character: CharacterType,
    },
    ReadyForGameStart,

    PlayerMovementInput {
    player_id: u8,
    direction: Direction,
    }, 
    NextLevelRequest,   

    RestartLevel,
    UndoLevel,
    ChangeLevelCheat(CurrentLevelIndex),

}

/// 服务器发送给客户端的消息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ServerMessage {
    /// 游戏状态改变通知
    StateChangeNotification(GameStates),
    /// 完整游戏状态同步
    FullStateSync(FullGameState),
    /// 玩家位置更新
    PlayerPositionUpdate { player_id: u8, position: Vec3 },
    //人物选择
    CharacterSelectionUpdate {
        player1_choice: Option<CharacterType>,
        player2_choice: Option<CharacterType>,
    },
    StartGameWithCharacters {
        player1_character: CharacterType,
        player2_character: CharacterType,
    },
    
    PlayerMovementUpdate {
        player_id: u8,
        direction: Direction
    },
    
    NextLevelNotification {
        level_index: CurrentLevelIndex,
    },

    DoRestartLevel,
    DoUndoLevel(Level),
    DoChangeLevel(CurrentLevelIndex),


}
/// 完整游戏状态
#[derive(Debug,Default,Clone, Serialize, Deserialize, Resource)]
pub struct FullGameState {
    pub current_state: GameStates,
    pub player1_position: Vec3,
    pub player2_position: Vec3,
    pub player1_character: CharacterType,
    pub player2_character: CharacterType,
    pub player1_logic_pos: (usize, usize),
    pub player2_logic_pos: (usize, usize),
    pub player1_bread: u32,
    pub player2_bread: u32,
    pub player1_can_move: bool,
    pub player2_can_move: bool,
    pub current_level:CurrentLevelIndex,
}

pub mod server;
pub mod client;

#[derive(Resource, Default)]
pub struct SelectionState {
    pub player1_ready: bool,
    pub player2_ready: bool,
    pub player1_choice: Option<CharacterType>,
    pub player2_choice: Option<CharacterType>,
}

#[derive(Event)]
pub struct RemotePlayerMove {
    pub player_id: u8,
    pub direction: Direction,
}
