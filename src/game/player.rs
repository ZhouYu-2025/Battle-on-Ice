use super::{
    audio::PlaySFX,
    level::{get_entity_on_logic_position, SymbolType::*, UpdateLevel},
    *,
};
use bevy::utils::Duration;
use crate::networking::ClientMessage;
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                player1_movement,
                player2_movement,
                component_animator_system::<Transform>,
                shake_other_ducks_in_direction,
                handle_remote_player_move,
            )
                .run_if(in_state(GameStates::Next)),
            
        )
        .add_event::<RemotePlayerMove>()
        .add_event::<ShakeOtherDucksInDir>();
    }
}

pub trait Duck {
    fn get_logic_position(&self) -> (usize, usize);
    fn get_bread_count(&self) -> u32;//面包计数
    fn can_move(&self) -> bool;
    //fn is_stuffed(&self) -> bool;
    fn set_logic_position(&mut self, position: (usize, usize));
    fn set_can_move(&mut self, can_move: bool);
    fn eat_bread(&mut self);
}

#[derive(Component)]
pub struct CommonDuck {
    pub logic_position: (usize, usize),
    pub can_move: bool, // stuffed_duck on breaking_ice => can't move
    pub bread_count: u32,
    //pub belly_capacity: u32,
}

impl Duck for CommonDuck {
    fn get_logic_position(&self) -> (usize, usize) {
        self.logic_position
    }

    fn get_bread_count(&self) -> u32 {
        self.bread_count
    }

    fn can_move(&self) -> bool {
        self.can_move
    }

    fn set_logic_position(&mut self, position: (usize, usize)) {
        self.logic_position = position;
    }

    fn set_can_move(&mut self, can_move: bool) {
        self.can_move = can_move;
    }

    fn eat_bread(&mut self) {
        self.bread_count += 1;
    }

    //fn is_stuffed(&self) -> bool {
    //    self.bread_count > 0
    //}
}

// the chosen duck
#[derive(Component)]
pub struct Player1;
#[derive(Component)]
pub struct Player2;
use renet::RenetClient;

/* 
//player movement
fn player1_movement(
    mut commands: Commands,
    // query
    mut player1_query: Query<
        (
            &mut Transform,
            &mut Sprite,
            &mut Handle<Image>,
            Option<&mut CommonDuck>,
            Entity,
        ),
        With<Player1>,
    >,
    // event
    mut events_sfx: EventWriter<PlaySFX>,
    mut events_update: EventWriter<UpdateLevel>,
    mut event_shake: EventWriter<ShakeOtherDucksInDir>,
    mut events_print: EventWriter<level::PrintLevel>,
    // resource
    key_board_input: Res<ButtonInput<KeyCode>>,
    level: ResMut<level::Level>,
    asset_server: Res<AssetServer>,
    audio_assets: Res<AudioAssets>,
    selected_characters:Res<SelectedCharacters>,
    mut client: Option<ResMut<RenetClient>>,

) {
    //player1
    if let Ok((transform, mut sprite, mut image, c_duck, entity)) = player1_query.get_single_mut() 
    {
        let duck: &mut dyn Duck = c_duck.unwrap().into_inner();

        if !duck.can_move() {
            info!("player1 cannot move");
            return;
        }

        let mut direction = utils::Direction::None;

        if key_board_input.just_pressed(KeyCode::KeyA)
        {
            info!("left");
            direction = utils::Direction::Left;
            sprite.flip_x = false;
        }

        if key_board_input.just_pressed(KeyCode::KeyD)
        {
            direction = utils::Direction::Right;
            sprite.flip_x = true;
        }
        if key_board_input.just_pressed(KeyCode::KeyW)
        {
            direction = utils::Direction::Up;
        }
        if key_board_input.just_pressed(KeyCode::KeyS)
        {
            direction = utils::Direction::Down;
        }
        
        if let Some(dir) = Some(direction).filter(|d| *d != utils::Direction::None) {
            if let Some(client) = &mut client {
                let message =
                    bincode::serialize(&ClientMessage::PlayerMovementInput{
                    player_id: 1,
                    direction: dir,
                },)
                    .unwrap();
                info!("player1 move:{:?}",dir);
                client.send_message(0, message);
            }
        }

        if direction != utils::Direction::None 
        {
            let duck_bread_count_before=duck.get_bread_count();
            let duck_can_move_before =duck.can_move();
            let end_position = slip(duck,direction,level);
            let duck_bread_count_after = duck.get_bread_count();
            let duck_can_move_after = duck.can_move();

            if duck_bread_count_after > duck_bread_count_before {
                //play eat sound
                events_sfx.send(PlaySFX {
                    source: audio_assets.eat.clone(),
                    volume: bevy::audio::Volume::new(0.05),
                });
            }

            if duck_bread_count_after > 0 {
                let texture_handle = match selected_characters.player1.unwrap_or(CharacterType::Duck) {
                    CharacterType::Duck => asset_server.load("sprites/stuffed_duck.png"),
                    CharacterType::Cat => asset_server.load("sprites/stuffed_cat.png"),
                    CharacterType::Bunny => asset_server.load("sprites/stuffed_bunny.png"),
                    CharacterType::Chick => asset_server.load("sprites/stuffed_chick.png"),
                };
                *image = texture_handle;            }

            if duck_can_move_before&&!duck_can_move_after
            {
                events_sfx.send(PlaySFX{
                    source : audio_assets.ice_breaking.clone(),
                    volume: bevy::audio::Volume::new(0.4),
                });
            } 

            //Update object positions
            duck.set_logic_position(end_position);
            //update the translation of ducks
            let v3 = logic_position_to_translation(end_position);
            let tween_translation = Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_millis(DUCK_MOVE_MILI_SECS),
                TransformPositionLens {
                    start: transform.translation,
                    end: Vec3::new(v3.x,v3.y,1.0),
                },
            )
            .with_repeat_count(1);
            
            // Scale the duck while moving
            let origin_scale = Vec3::new(1.0 * RESIZE, 1.0 * RESIZE, 1.0);
            let new_scale = transform.scale * Vec3::new(1.3, 0.7, 1.);
            let tween_scale = Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_millis(DUCK_MOVE_MILI_SECS),
                TransformScaleLens {
                    start: new_scale,
                    end: origin_scale,
                },
            )
            .with_repeat_count(1);

            let track: Tracks<Transform> = Tracks::new(vec![tween_translation, tween_scale]);

            commands.entity(entity).insert(Animator::new(track));
            event_shake.send(ShakeOtherDucksInDir {
                direction,
                player_logic_position: duck.get_logic_position(),
            });

            // play quark sound
            events_sfx.send(PlaySFX {
                source: audio_assets.quark.clone(),
                volume: bevy::audio::Volume::new(0.4),
            });
            events_print.send(level::PrintLevel);
            events_update.send(UpdateLevel); 
        }    
    }
}


fn player2_movement(
    mut commands: Commands,
    // query
    mut player2_query: Query<
        (
            &mut Transform,
            &mut Sprite,
            &mut Handle<Image>,
            Option<&mut CommonDuck>,
            Entity,
        ),
        With<Player2>,
    >,
    // event
    mut events_sfx: EventWriter<PlaySFX>,
    mut events_update: EventWriter<UpdateLevel>,
    mut event_shake: EventWriter<ShakeOtherDucksInDir>,
    mut events_print: EventWriter<level::PrintLevel>,
    // resource
    key_board_input: Res<ButtonInput<KeyCode>>,
    level: ResMut<level::Level>,
    asset_server: Res<AssetServer>,
    audio_assets: Res<AudioAssets>,
    selected_characters:Res<SelectedCharacters>,
    mut client: Option<ResMut<RenetClient>>,

) {
    //player2
    if let Ok((transform, mut sprite, mut image, c_duck, entity)) = player2_query.get_single_mut() 
    {
        let duck: &mut dyn Duck = c_duck.unwrap().into_inner();

        if !duck.can_move() {
            return;
        }
        let mut direction = utils::Direction::None;

        if key_board_input.just_pressed(KeyCode::ArrowLeft)
        {
            direction = utils::Direction::Left;
            sprite.flip_x = false;
        }
        if key_board_input.just_pressed(KeyCode::ArrowRight)
        {
            direction = utils::Direction::Right;
            sprite.flip_x = true;
        }
        if key_board_input.just_pressed(KeyCode::ArrowUp)
        {
            direction = utils::Direction::Up;
        }
        if key_board_input.just_pressed(KeyCode::ArrowDown)
        {
            direction = utils::Direction::Down;
        }

        if let Some(dir) = Some(direction).filter(|d| *d != utils::Direction::None) {
            if let Some(client) = &mut client {
                let message =
                    bincode::serialize(&ClientMessage::PlayerMovementInput{
                    player_id: 2,
                    direction: dir,
                },)
                    .unwrap();
                info!("player2 move:{:?}",dir);
                client.send_message(0, message);
            }
        }


        if direction != utils::Direction::None 
        {
            let duck_bread_count_before=duck.get_bread_count();
            let duck_can_move_before =duck.can_move();
            let end_position = slip(duck,direction,level);
            let duck_bread_count_after = duck.get_bread_count();
            let duck_can_move_after = duck.can_move();

            if duck_bread_count_after > duck_bread_count_before {
                //play eat sound
                events_sfx.send(PlaySFX {
                    source: audio_assets.eat.clone(),
                    volume: bevy::audio::Volume::new(0.05),
                });
            }

            if duck_bread_count_after > 0 {
                let texture_handle = match selected_characters.player2.unwrap_or(CharacterType::Duck) {
                    CharacterType::Duck => asset_server.load("sprites/stuffed_duck.png"),
                    CharacterType::Cat => asset_server.load("sprites/stuffed_cat.png"),
                    CharacterType::Bunny => asset_server.load("sprites/stuffed_bunny.png"),
                    CharacterType::Chick => asset_server.load("sprites/stuffed_chick.png"),
                };
                *image = texture_handle;            }

            if duck_can_move_before&&!duck_can_move_after
            {
                events_sfx.send(PlaySFX{
                    source : audio_assets.ice_breaking.clone(),
                    volume: bevy::audio::Volume::new(0.4),
                });
            } 

            //Update object positions
            duck.set_logic_position(end_position);
            //update the translation of ducks
            let v3 = logic_position_to_translation(end_position);
            let tween_translation = Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_millis(DUCK_MOVE_MILI_SECS),
                TransformPositionLens {
                    start: transform.translation,
                    end: Vec3::new(v3.x,v3.y,1.0),
                },
            )
            .with_repeat_count(1);
            
            // Scale the duck while moving
            let origin_scale = Vec3::new(1.0 * RESIZE, 1.0 * RESIZE, 1.0);
            let new_scale = transform.scale * Vec3::new(1.3, 0.7, 1.);
            let tween_scale = Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_millis(DUCK_MOVE_MILI_SECS),
                TransformScaleLens {
                    start: new_scale,
                    end: origin_scale,
                },
            )
            .with_repeat_count(1);

            let track: Tracks<Transform> = Tracks::new(vec![tween_translation, tween_scale]);

            commands.entity(entity).insert(Animator::new(track));
            event_shake.send(ShakeOtherDucksInDir {
                direction,
                player_logic_position: duck.get_logic_position(),
            });
            //let v3 = logic_position_to_translation(end_position, window_query.get_single().unwrap());
            //transform.translation = Vec3::new(v3.x, v3.y, 1.0);

            // play quark sound
            events_sfx.send(PlaySFX {
                source: audio_assets.quark.clone(),
                volume: bevy::audio::Volume::new(0.4),
            });
            events_print.send(level::PrintLevel);
            events_update.send(UpdateLevel); 
        }    
    }
}
*/

fn player1_movement(
    key_board_input: Res<ButtonInput<KeyCode>>,
    mut client: Option<ResMut<RenetClient>>,
) {
    let mut direction = utils::Direction::None;

    if key_board_input.just_pressed(KeyCode::KeyA) {
        direction = utils::Direction::Left;
    }
    if key_board_input.just_pressed(KeyCode::KeyD) {
        direction = utils::Direction::Right;
    }
    if key_board_input.just_pressed(KeyCode::KeyW) {
        direction = utils::Direction::Up;
    }
    if key_board_input.just_pressed(KeyCode::KeyS) {
        direction = utils::Direction::Down;
    }

    if let Some(dir) = Some(direction).filter(|d| *d != utils::Direction::None) {
        if let Some(client) = &mut client {
            let message = bincode::serialize(&ClientMessage::PlayerMovementInput {
                player_id: 1,
                direction: dir,
            }).unwrap();
            client.send_message(0, message);
            info!("Sent Player1 input: {:?}", dir);
        }
    }
}


fn player2_movement(
    key_board_input: Res<ButtonInput<KeyCode>>,
    mut client: Option<ResMut<RenetClient>>,
) {
    let mut direction = utils::Direction::None;

    if key_board_input.just_pressed(KeyCode::ArrowLeft) {
        direction = utils::Direction::Left;
    }
    if key_board_input.just_pressed(KeyCode::ArrowRight) {
        direction = utils::Direction::Right;
    }
    if key_board_input.just_pressed(KeyCode::ArrowUp) {
        direction = utils::Direction::Up;
    }
    if key_board_input.just_pressed(KeyCode::ArrowDown) {
        direction = utils::Direction::Down;
    }

    if let Some(dir) = Some(direction).filter(|d| *d != utils::Direction::None) {
        if let Some(client) = &mut client {
            let message = bincode::serialize(&ClientMessage::PlayerMovementInput {
                player_id: 2,
                direction: dir,
            }).unwrap();
            client.send_message(0, message);
            info!("Sent Player2 input: {:?}", dir);
        }
    }
}



// Slip until hitting the wall or bread
// common duck
pub fn slip(
    duck: &mut dyn Duck,
    direction: utils::Direction,
    // resource
    mut level: ResMut<level::Level>,
) -> (usize, usize) {
    // Up: row--, Down: row++, Left: col--, Right: col++
    let rows = level.0.len();
    let logic_position = duck.get_logic_position();
    let cols = level.0[logic_position.0].len();
    let mut position = logic_position;
    match direction {
        utils::Direction::Up => {
            while position.0 > 0 && is_valid_move(level.0[position.0 - 1][position.1], duck) {
                position.0 -= 1;
                if collide_with_object(level.0[position.0][position.1], duck) {
                    break;
                }
            }
        }
        utils::Direction::Down => {
            while position.0 < rows - 1 && is_valid_move(level.0[position.0 + 1][position.1], duck)
            {
                position.0 += 1;
                if collide_with_object(level.0[position.0][position.1], duck) {
                    break;
                }
            }
        }
        utils::Direction::Left => {
            while position.1 > 0 && is_valid_move(level.0[position.0][position.1 - 1], duck) {
                position.1 -= 1;
                if collide_with_object(level.0[position.0][position.1], duck) {
                    break;
                }
            }
        }
        utils::Direction::Right => {
            while position.1 < cols - 1 && is_valid_move(level.0[position.0][position.1 + 1], duck)
            {
                position.1 += 1;
                if collide_with_object(level.0[position.0][position.1], duck) {
                    break;
                }
            }
        }
        _ => (),
    }

    // Update symbols on the level
    let mut duck_char: char = DuckOnIce.get_symbol();
    if duck.get_bread_count() > 0 {
        duck_char = StuffedDuckOnIce.get_symbol();
    }

    if level.0[logic_position.0][logic_position.1] == DuckOnBreakingIce.get_symbol() {
        level.0[logic_position.0][logic_position.1] = BreakingIce.get_symbol();
    } else {
        level.0[logic_position.0][logic_position.1] = Ice.get_symbol();
    }
    if level.0[position.0][position.1] == BreakingIce.get_symbol() {
        level.0[position.0][position.1] = DuckOnBreakingIce.get_symbol();
    } else {
        level.0[position.0][position.1] = duck_char;
    }
    if !duck.can_move() {
        level.0[position.0][position.1] = DuckOnWater.get_symbol();
    }
    position
}

fn is_valid_move(symbol: char, duck: &dyn Duck) -> bool {
    symbol != Wall.get_symbol()
        && symbol != DuckOnIce.get_symbol()
        && symbol != DuckOnWater.get_symbol()
        && symbol != DuckOnBreakingIce.get_symbol()
        && symbol != StuffedDuckOnIce.get_symbol()
        && (!duck.get_bread_count()>0 || symbol != BreadOnIce.get_symbol())
}

// TODO: replace it with eat_bread_or_break_ice
fn collide_with_object(symbol: char, duck: &mut dyn Duck) -> bool {
    if symbol == BreadOnIce.get_symbol() {
        duck.eat_bread();
        return true;
    }
    if symbol == BreakingIce.get_symbol() {
        duck.set_can_move(false);
        return true;
    }
    false
}

#[derive(Event)]
pub struct ShakeOtherDucksInDir {
    direction: utils::Direction,
    player_logic_position: (usize, usize),
}

pub fn shake_other_ducks_in_direction(
    mut commands: Commands,
    level: Res<level::Level>,
    query: Query<(Entity, &Transform), With<level::Object>>,
    mut events: EventReader<ShakeOtherDucksInDir>,
) {
    for e in events.read() {
        let direction = e.direction;
        let mut ducks_to_shake: Vec<Entity> = Vec::new();
        let mut position = e.player_logic_position;
        let rows = level.0.len();
        let cols = level.0[position.0].len();

        let delta: (i32, i32) = match direction {
            utils::Direction::Up => (-1, 0),
            utils::Direction::Down => (1, 0),
            utils::Direction::Left => (0, -1),
            utils::Direction::Right => (0, 1),
            utils::Direction::None => return,
        };

        while position.0 > 0 && position.0 < rows - 1 && position.1 > 0 && position.1 < cols - 1 {
            position.0 = (delta.0 + position.0 as i32) as usize;
            position.1 = (delta.1 + position.1 as i32) as usize;
            let symbol = level.0[position.0][position.1];
            if [
                DuckOnBreakingIce.get_symbol(),
                DuckOnIce.get_symbol(),
                DuckOnWater.get_symbol(),
                StuffedDuckOnIce.get_symbol(),
            ]
            .contains(&symbol)
            {
                if let Some(entity) = get_entity_on_logic_position(position, &query) {
                    ducks_to_shake.push(entity);
                }
            } else {
                break;
            }
        }

        for entity in ducks_to_shake {
            let origin_scale = Vec3::new(1.0 * RESIZE, 1.0 * RESIZE, 1.0);
            let new_scale = origin_scale * Vec3::new(1.3, 0.7, 1.);
            let tween_scale = Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_millis(300),
                TransformScaleLens {
                    start: new_scale,
                    end: origin_scale,
                },
            )
            .with_repeat_count(1);
            let delay = Delay::new(Duration::from_millis(DUCK_MOVE_MILI_SECS));
            commands
                .entity(entity)
                .insert(Animator::new(delay.then(tween_scale)));
        }
    }
}

use crate::networking::RemotePlayerMove;
pub fn handle_remote_player_move(
    mut commands: Commands,
    mut move_events: EventReader<RemotePlayerMove>,
    mut player1_query: Query<(
        &mut Transform,
        &mut Sprite,
        &mut Handle<Image>,
        Option<&mut CommonDuck>,
        Entity,
    ), (With<Player1>,Without<Player2>)>,
    mut player2_query: Query<(
        &mut Transform,
        &mut Sprite,
        &mut Handle<Image>,
        Option<&mut CommonDuck>,
        Entity,
    ), (With<Player2>,Without<Player1>)>,
    mut events_sfx: EventWriter<PlaySFX>,
    mut events_update: EventWriter<UpdateLevel>,
    mut event_shake: EventWriter<ShakeOtherDucksInDir>,
    mut events_print: EventWriter<level::PrintLevel>,
    level: ResMut<level::Level>,
    asset_server: Res<AssetServer>,
    audio_assets: Res<AudioAssets>,
    selected_characters: Res<SelectedCharacters>,
) {
    if let Some(RemotePlayerMove { player_id, direction }) = move_events.read().next() {
        let query = match player_id {
            1 => player1_query.get_single_mut(),
            2 => player2_query.get_single_mut(),
            _ => return,
        };

        if let Ok((transform, mut sprite, mut image, c_duck, entity)) = query {
            let duck: &mut dyn Duck = c_duck.unwrap().into_inner();

            if !duck.can_move() {
                return;
            }

            // Flip sprite if needed
            match direction {
                utils::Direction::Left => sprite.flip_x = false,
                utils::Direction::Right => sprite.flip_x = true,
                _ => {}
            }

            let before = duck.get_bread_count();
            let before_move = duck.can_move();

            let end_position = slip(duck, *direction, level);
            let after = duck.get_bread_count();
            let after_move = duck.can_move();

            if after > before {
                events_sfx.send(PlaySFX {
                    source: audio_assets.eat.clone(),
                    volume: bevy::audio::Volume::new(0.05),
                });
            }

            if after > 0 {
                let texture_handle = match player_id {
                    1 => match selected_characters.player1.unwrap_or(CharacterType::Duck) {
                        CharacterType::Duck => asset_server.load("sprites/stuffed_duck.png"),
                        CharacterType::Cat => asset_server.load("sprites/stuffed_cat.png"),
                        CharacterType::Bunny => asset_server.load("sprites/stuffed_bunny.png"),
                        CharacterType::Chick => asset_server.load("sprites/stuffed_chick.png"),
                    },
                    2 => match selected_characters.player2.unwrap_or(CharacterType::Duck) {
                        CharacterType::Duck => asset_server.load("sprites/stuffed_duck.png"),
                        CharacterType::Cat => asset_server.load("sprites/stuffed_cat.png"),
                        CharacterType::Bunny => asset_server.load("sprites/stuffed_bunny.png"),
                        CharacterType::Chick => asset_server.load("sprites/stuffed_chick.png"),
                    },
                    _=> return,
                };
                *image = texture_handle;
            }

            if before_move && !after_move {
                events_sfx.send(PlaySFX {
                    source: audio_assets.ice_breaking.clone(),
                    volume: bevy::audio::Volume::new(0.4),
                });
            }

            duck.set_logic_position(end_position);
            let v3 = logic_position_to_translation(end_position);

            let tween_translation = Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_millis(DUCK_MOVE_MILI_SECS),
                TransformPositionLens {
                    start: transform.translation,
                    end: Vec3::new(v3.x, v3.y, 1.0),
                },
            )
            .with_repeat_count(1);

            let origin_scale = Vec3::new(1.0 * RESIZE, 1.0 * RESIZE, 1.0);
            let new_scale = transform.scale * Vec3::new(1.3, 0.7, 1.);
            let tween_scale = Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_millis(DUCK_MOVE_MILI_SECS),
                TransformScaleLens {
                    start: new_scale,
                    end: origin_scale,
                },
            )
            .with_repeat_count(1);

            let track: Tracks<Transform> = Tracks::new(vec![tween_translation, tween_scale]);
            commands.entity(entity).insert(Animator::new(track));

            event_shake.send(ShakeOtherDucksInDir {
                direction: *direction,
                player_logic_position: duck.get_logic_position(),
            });

            events_sfx.send(PlaySFX {
                source: audio_assets.quark.clone(),
                volume: bevy::audio::Volume::new(0.4),
            });
            events_print.send(level::PrintLevel);
            events_update.send(UpdateLevel);
        }
    }
}
