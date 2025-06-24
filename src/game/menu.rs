use bevy::prelude::*;
use super::*;
use crate::game::ui::GameHints;
use crate::networking::{ClientMessage};

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameStates::GameMenu), setup_menu)
           .add_systems(Update, menu_interaction.run_if(in_state(GameStates::GameMenu)))
           .add_systems(OnExit(GameStates::GameMenu), cleanup_menu);
    }
}

#[derive(Component)]
struct MenuEntity;

#[derive(Component)]
pub struct PlayButton;

const NORMAL_BUTTON: Color = MY_ORANGE;
const HOVERED_BUTTON: Color = Color::srgb(222.0/255.0 + 0.1, 112.0/255.0 + 0.1, 40.0/255.0 + 0.1);
const PRESSED_BUTTON: Color = Color::srgb(0.75, 0.75, 0.75);

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 游戏标题
    commands.spawn((
        TextBundle::from_section(
            "BATTLE on ICE!",
            TextStyle {
                font: asset_server.load("fonts/NotJamChunky8.ttf"),
                font_size: 80.0,
                color: MY_ORANGE,
            },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(150.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            ..default()
        }),
        MenuEntity,
    ));

    // Play按钮
    commands.spawn((
        ButtonBundle {
            style: Style {
                width: Val::Px(300.0),
                height: Val::Px(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-150.0), //x
                    top: Val::Px(-50.0),   //y
                    ..default()
                },
                ..default()
            },
            background_color: NORMAL_BUTTON.into(),
            ..default()
        },
        PlayButton,
        MenuEntity,
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "Play Game",
            TextStyle {
                font: asset_server.load("fonts/NotJamChunky8.ttf"),
                font_size: 40.0,
                color: Color::WHITE,
            },
        )
    );
    });
}

use renet::RenetClient;
fn menu_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<PlayButton>)
    >,
    mut next_state: ResMut<NextState<GameStates>>,
    mut client: Option<ResMut<RenetClient>>,
) 
{
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
// If we're the client, send button press to server

                if let Some(client) = &mut client {
                    let message = bincode::serialize(&ClientMessage::StateChangeRequest(
                        GameStates::CharacterSelection
                    )).unwrap();
                    info!("send state change to menu->selection");
                    //next_state.set(GameStates::CharacterSelection);
                    client.send_message(0, message); // 使用可靠通道
                } else {
                    // 如果是服务器，直接处理
                    info!("handle menu-selection");
                    next_state.set(GameStates::CharacterSelection);
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

fn cleanup_menu(
    mut commands: Commands, 
    query: Query<Entity, With<MenuEntity>>,
    level_entities: Query<Entity, With<level::Object>>,
    hints_query: Query<Entity, With<GameHints>>
) {
    // 清理菜单实体
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    
    // 额外清理可能残留的关卡实体
    for entity in level_entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
    
    // 清理提示
    for hint_entity in hints_query.iter() {
        commands.entity(hint_entity).despawn();
    }
    
}