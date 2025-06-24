use bevy::prelude::*;
use super::*;
use crate::game::level::CurrentLevelIndex;
use crate::game::level::Level;
use crate::game::ui::GameHints;
use crate::game::ui::LevelTitle;
use crate::game::ui::StuffedDucksCount;

const NORMAL_BUTTON: Color = MY_ORANGE;
const HOVERED_BUTTON: Color = Color::srgb(1.0, 0.6, 0.2);
const PRESSED_BUTTON: Color = Color::srgb(0.75, 0.75, 0.75);

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameStates::Celebration), setup_celebration)
           .add_systems(Update, (
               character_animation,
               menu_button_interaction,
               exit_game,
           ).run_if(in_state(GameStates::Celebration)))
           .add_systems(OnExit(GameStates::Celebration), cleanup_celebration);
    }
}

#[derive(Component)]
struct CelebrationEntity;

#[derive(Component)]
struct MenuButton;

#[derive(Component)]
struct ExitButton;

#[derive(Component)]
struct CornerCharacter;

#[derive(Component)]
struct CelebratingCharacter {
    timer: Timer,
    is_stuffed: bool,
    character_type: CharacterType,
}


#[derive(Component)]
struct ButtonContainer;

#[derive(Bundle)]
struct CharacterBundle {
    image: ImageBundle,
    character: CelebratingCharacter,
    marker: CornerCharacter,
}

fn setup_celebration(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    image_assets: Res<ImageAssets>,
) {
    // 半透明背景
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            background_color: Color::srgb(0.0, 0.0, 0.0).into(),
            ..default()
        },
        CelebrationEntity,
    ));

    // 大标题居中
    commands.spawn((
        TextBundle::from_section(
            "CONGRATULATIONS!",
            TextStyle {
                font: asset_server.load("fonts/NotJamChunky8.ttf"),
                font_size: 60.0,
                color: MY_ORANGE,
            },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(160.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            ..default()
        }),
        CelebrationEntity,
    ));

    // 四个角的角色
    let characters = [
        (Val::Percent(5.0), Val::Percent(5.0), CharacterType::Duck),//左下
        (Val::Percent(80.0), Val::Percent(5.0), CharacterType::Cat),//右下
        (Val::Percent(5.0), Val::Percent(70.0), CharacterType::Bunny),//左上
        (Val::Percent(80.0), Val::Percent(70.0), CharacterType::Chick),//右上
    ];

    for (left, bottom, char_type) in characters {
        let texture = match char_type {
            CharacterType::Duck => image_assets.duck.clone(),
            CharacterType::Cat => image_assets.cat.clone(),
            CharacterType::Bunny => image_assets.bunny.clone(),
            CharacterType::Chick => image_assets.chick.clone(),
        };

        commands.spawn(CharacterBundle {
            image: ImageBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left,
                    bottom,
                    width: Val::Px(150.0), 
                    height: Val::Px(150.0),
                    ..default()
                },
                image: UiImage::new(texture),
                ..default()
            },
            character: CelebratingCharacter {
                timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                is_stuffed: false,
                character_type: char_type,
            },
            marker: CornerCharacter,
        }).insert(CelebrationEntity);
    }

    // 按钮容器 - 居中
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                margin: UiRect {
                    left: Val::Px(-150.0), //x
                    top: Val::Px(-50.0),   //y
                    ..default()
                },
                ..default()
            },
            ..default()
        },
        ButtonContainer,
        CelebrationEntity,
    )).with_children(|parent| {
        // 退出游戏按钮
        parent.spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(300.0),
                    height: Val::Px(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            ExitButton,
        )).with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Exit Game",
                TextStyle {
                    font: asset_server.load("fonts/NotJamChunky8.ttf"),
                    font_size: 24.0,
                    color: Color::WHITE,
                },
            ));
        });

        // 主菜单按钮
        parent.spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(300.0),
                    height: Val::Px(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            MenuButton,
        )).with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Main Menu",
                TextStyle {
                    font: asset_server.load("fonts/NotJamChunky8.ttf"),
                    font_size: 24.0,
                    color: Color::WHITE,
                },
            ));
        });
    });
}

fn character_animation(
    time: Res<Time>,
    image_assets: Res<ImageAssets>,
    mut query: Query<(&mut CelebratingCharacter, &mut UiImage)>,
) {
    for (mut character, mut ui_image) in query.iter_mut() {
        character.timer.tick(time.delta());
        if character.timer.just_finished() {
            character.is_stuffed = !character.is_stuffed;
            
            ui_image.texture = match (character.character_type, character.is_stuffed) {
                (CharacterType::Duck, false) => image_assets.duck.clone(),
                (CharacterType::Duck, true) => image_assets.stuffed_duck.clone(),
                (CharacterType::Cat, false) => image_assets.cat.clone(),
                (CharacterType::Cat, true) => image_assets.stuffed_cat.clone(),
                (CharacterType::Bunny, false) => image_assets.bunny.clone(),
                (CharacterType::Bunny, true) => image_assets.stuffed_bunny.clone(),
                (CharacterType::Chick, false) => image_assets.chick.clone(),
                (CharacterType::Chick, true) => image_assets.stuffed_chick.clone(),
            };
        }
    }
}

use renet::RenetClient;
use crate::networking::ClientMessage;

fn menu_button_interaction(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<MenuButton>)>,
    //mut level_index: ResMut<CurrentLevelIndex>,
    //mut commands: Commands,
    //mut next_state: ResMut<NextState<GameStates>>,
    mut client: Option<ResMut<RenetClient>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                
                // 重置关卡索引
                if let Some(client) = &mut client {
                    let message = bincode::serialize(&ClientMessage::StateChangeRequest(
                        GameStates::GameMenu
                    )).unwrap();
                    client.send_message(0, message); // 使用可靠通道
                } 

            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}
// 修改为退出游戏功能
fn exit_game(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<ExitButton>)>,
    mut app_exit_events: EventWriter<bevy::app::AppExit>,
    //mut client: Option<ResMut<RenetClient>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
// If we're the client, send exit button press
                
                app_exit_events.send(bevy::app::AppExit::Success);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}
use crate::game::ui::MutUI;
use crate::game::selection::SelectionUI;
fn cleanup_celebration(
    mut commands: Commands,
    query: Query<Entity, With<CelebrationEntity>>,
    level_entities: Query<Entity, With<level::Object>>,
    game_hints: Query<Entity, With<GameHints>>,
    level_titles: Query<Entity, With<LevelTitle>>,
    stuffed_ducks: Query<Entity, With<StuffedDucksCount>>,
    selectionui: Query<Entity, With<SelectionUI>>,
    ui_query: Query<Entity, With<MutUI>>,


) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // 清理可能残留的关卡实体
    for entity in level_entities.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // 清理游戏提示
    for hint_entity in game_hints.iter() {
        commands.entity(hint_entity).despawn();
    }
    
    // 清理关卡标题
    for title_entity in level_titles.iter() {
        commands.entity(title_entity).despawn();
    }
    
    // 清理面包计数UI
    for duck_entity in stuffed_ducks.iter() {
        commands.entity(duck_entity).despawn();
    }
    
    //清理ui界面
    for entity in ui_query.iter() {
        commands.entity(entity).despawn();
    }

    for entity in selectionui.iter() {
        commands.entity(entity).despawn_recursive();
    }
    // 确保移除关卡资源
    commands.remove_resource::<Level>(); 
}