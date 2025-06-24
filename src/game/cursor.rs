use super::{player::CommonDuck, *};
use crate::game::player::{Player1,Player2};
use bevy::{input::mouse::MouseButtonInput, window::PrimaryWindow};
use crate::game::{CharacterType,SelectedCharacters};
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorPosition>()
            .add_systems(
                Update,
                get_cursor_position.run_if(in_state(GameStates::Next)),
            )
            .add_systems(Update, click_detection.run_if(in_state(GameStates::Next)));
    }
}

const DISTANCE: f32 = (640.0 / 2.0 + 5.0) * RESIZE;

#[derive(Component)]
pub struct ArrowHint;

#[derive(Resource, Default)]
pub struct CursorPosition(pub Vec2);

fn get_cursor_position(
    mut commands: Commands,
    // resource
    image_assets: Res<ImageAssets>,
    mut cursor_position: ResMut<CursorPosition>,
    // query
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    duck_query: Query<(&CommonDuck, &Transform), (With<CommonDuck>, Without<Player1>,Without<Player2>)>,
    arrow_query: Query<Entity, (With<ArrowHint>, Without<Parent>)>,
) {
    let (camera, camera_transform) = camera_query.single();
    let window = window_query.get_single().unwrap();
    if let Some(cursor_pos) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    {
        cursor_position.0 = cursor_pos;

        for entity in arrow_query.iter() {
            commands.entity(entity).despawn();
        }
        // Hover cursor on the duck, show arrow hint
        for (_, transform) in duck_query.iter() {
            //let duck_position_v3 = logic_position_to_translation(duck.logic_position);
            let duck_position_v3=transform.translation;
            let duck_position: Vec2 = Vec2 {
                x: duck_position_v3.x,
                y: duck_position_v3.y,
            };
            if (cursor_position.0 - duck_position).length() < DISTANCE {
                commands.spawn((
                    SpriteBundle {
                        transform: Transform {
                            translation: Vec3::new(
                                duck_position.x,
                                duck_position.y + SPRITE_SIZE,
                                2.0,
                            ),
                            rotation: Quat::IDENTITY,
                            scale: Vec3::new(1.0 * RESIZE, 1.0 * RESIZE, 1.0),
                        },
                        texture: image_assets.arrow.clone(),
                        ..default()
                    },
                    ArrowHint,
                    //level::Object,
                ));
            }
        }
    }
}


#[derive(Component)]
pub struct CharacterTag(pub CharacterType);

pub fn click_detection(
    mut commands: Commands,
    // event
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    // query
    duck_query: Query<(&CommonDuck, &CharacterTag,Entity), (With<CommonDuck>, Without<Player1>,Without<Player2>)>,
    player1_query: Query<Entity, With<Player1>>,
    player2_query: Query<Entity, With<Player2>>,
    arrow_hint_query: Query<Entity, (With<ArrowHint>, With<Parent>)>,
    // resource
    cursor_position: Res<CursorPosition>,
    image_assets: Res<ImageAssets>,
    mut selected_characters: ResMut<SelectedCharacters>,
) {
    for event in mouse_button_input_events.read() {
        
        //player1
        if event.button ==MouseButton::Left {
            for (duck, charactertag,entity) in duck_query.iter() {
                let duck_position_v3 = logic_position_to_translation(duck.logic_position);
                let duck_position: Vec2 = Vec2 {
                    x: duck_position_v3.x,
                    y: duck_position_v3.y,
                };
                if (cursor_position.0 - duck_position).length() < DISTANCE{
                    selected_characters.player1 = Some(charactertag.0);
                    commands
                        .entity(entity)
                        .insert(Player1)
                        .with_children(|parent| {
                            parent.spawn((
                                SpriteBundle {
                                    transform: Transform {
                                        translation: Vec3::new(0.0, 500.0, 1.0),
                                        ..default()
                                    },
                                    texture: image_assets.arrow2.clone(),
                                    ..default()
                                },
                                ArrowHint,
                                level::Object,
                            ));
                        });
                    // Clear the previous player1
                    for entity in player1_query.iter() {
                        commands.entity(entity).remove::<Player1>();
                        commands.entity(entity).clear_children();
                    }
                    // Clear the previous arrow hint
                    for entity in arrow_hint_query.iter() {
                        commands.entity(entity).despawn();
                    }
                }
            }
        }
            
        if event.button == MouseButton::Right{
            for (duck, charactertag,entity) in duck_query.iter() {
                let duck_position_v3 = logic_position_to_translation(duck.logic_position);
                let duck_position: Vec2 = Vec2 {
                    x: duck_position_v3.x,
                    y: duck_position_v3.y,
                };
                if (cursor_position.0 - duck_position).length() < DISTANCE{
                    selected_characters.player2 = Some(charactertag.0);
                    commands
                        .entity(entity)
                        .insert(Player2)
                        .with_children(|parent| {
                            parent.spawn((
                                SpriteBundle {
                                    transform: Transform {
                                        translation: Vec3::new(0.0, 500.0, 1.0),
                                        ..default()
                                    },
                                    texture: image_assets.arrow.clone(),
                                    ..default()
                                },
                                ArrowHint,
                                level::Object,
                            ));
                        });
                    // Clear the previous player
                    for entity in player2_query.iter() {
                        commands.entity(entity).remove::<Player2>();
                        commands.entity(entity).clear_children();
                    }
                    // Clear the previous arrow hint
                    for entity in arrow_hint_query.iter() {
                        commands.entity(entity).despawn();
                    }
                }
            }
        }
    }
}
