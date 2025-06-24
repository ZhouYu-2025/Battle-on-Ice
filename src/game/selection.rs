use bevy::{prelude::*, input::ButtonInput};
use super::{CharacterType, GameStates, ImageAssets, SelectedCharacters, MY_ORANGE};

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameStates::CharacterSelection), setup_selection_ui)
           .add_systems(Update, (
               handle_character_selection,
               button_visual_feedback,
               exit_selection,
           ).run_if(in_state(GameStates::CharacterSelection)))
           .add_systems(OnExit(GameStates::CharacterSelection), cleanup_selection_ui);
    }
}

#[derive(Component)]
pub struct SelectionUI;

#[derive(Component)]
struct CharacterButton(CharacterType);

const NORMAL_BUTTON: Color = MY_ORANGE;
const HOVERED_BUTTON: Color = Color::srgb(1.0, 0.6, 0.2);
const PRESSED_BUTTON: Color = Color::srgb(0.75, 0.75, 0.75);

const BUTTON_SIZE: Val = Val::Px(150.0);
const BUTTON_SPACING: Val = Val::Px(30.0);
const CHARACTER_IMAGE_SIZE: Val = Val::Px(200.0);

fn setup_selection_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    image_assets: Res<ImageAssets>,
) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                padding: UiRect::top(Val::Px(20.0)),
                ..default()
            },
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.8).into(),
            ..default()
        },
        SelectionUI,
    )).with_children(|parent| {

        parent.spawn(TextBundle {
            style: Style {
                margin: UiRect::top(Val::Px(40.0)),
                ..default()
            },
            text: Text::from_section(
                "Choose Your Spirit (Left-P1, Right-P2)",
                TextStyle {
                    font: asset_server.load("fonts/NotJamChunky8.ttf"),
                    font_size: 40.0,
                    color: Color::WHITE,
                },
            ),
            ..default()
        });
        
        //character picture and button
        parent.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                margin: UiRect::top(Val::Px(20.0)),
                ..default()
            },
            ..default()
        }).with_children(|parent| {
            parent.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: BUTTON_SPACING,
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
                ..default()
            }).with_children(|parent| {
                spawn_character_image(parent, &image_assets, CharacterType::Duck);
                spawn_character_image(parent, &image_assets, CharacterType::Cat);
                spawn_character_image(parent, &image_assets, CharacterType::Bunny);
                spawn_character_image(parent, &image_assets, CharacterType::Chick);
            });

            parent.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: BUTTON_SPACING,
                    ..default()
                },
                ..default()
            }).with_children(|parent| {
                spawn_character_button(parent, &asset_server, CharacterType::Duck);
                spawn_character_button(parent, &asset_server, CharacterType::Cat);
                spawn_character_button(parent, &asset_server, CharacterType::Bunny);
                spawn_character_button(parent, &asset_server, CharacterType::Chick);
            });


        });
        
        parent.spawn(TextBundle {
            style: Style {
                margin: UiRect::bottom(Val::Px(30.0)),
                ..default()
            },
            text: Text::from_section(
                "Press ENTER to start game",
                TextStyle {
                    font: asset_server.load("fonts/NotJamChunky8.ttf"),
                    font_size: 24.0,
                    color: Color::WHITE,
                },
            ),
            ..default()
        });

    });
}

fn spawn_character_image(parent: &mut ChildBuilder, image_assets: &Res<ImageAssets>, char_type: CharacterType) {
    let texture = match char_type {
        CharacterType::Duck => image_assets.duck.clone(),
        CharacterType::Cat => image_assets.cat.clone(),
        CharacterType::Bunny => image_assets.bunny.clone(),
        CharacterType::Chick => image_assets.chick.clone(),
    };

    parent.spawn(ImageBundle {
        style: Style {
            width: CHARACTER_IMAGE_SIZE,
            height: CHARACTER_IMAGE_SIZE,
            ..default()
        },
        image: UiImage::new(texture),
        ..default()
    });
}

fn spawn_character_button(parent: &mut ChildBuilder, asset_server: &Res<AssetServer>, char_type: CharacterType) {
    parent.spawn((
        ButtonBundle {
            style: Style {
                width: BUTTON_SIZE,
                height: BUTTON_SIZE,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: NORMAL_BUTTON.into(),
            ..default()
        },
        CharacterButton(char_type),
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            match char_type {
                CharacterType::Duck => "Duck",
                CharacterType::Cat => "Cat",
                CharacterType::Chick => "Chick",
                CharacterType::Bunny => "Bunny",
            },
            TextStyle {
                font: asset_server.load("fonts/NotJamChunky8.ttf"),
                font_size: 24.0,
                color: Color::WHITE,
            },
        ));
    });
}

use bevy::input::mouse::MouseButtonInput;


// 修改 handle_character_selection 函数：

fn handle_character_selection(
    mut events: EventReader<MouseButtonInput>,
    buttons: Query<(&Interaction, &CharacterButton), With<Button>>,
    mut selected: ResMut<SelectedCharacters>,
) {
    for event in events.read() {
        for (interaction, character_btn) in buttons.iter() {
            if *interaction == Interaction::Hovered {
                match event.button {
                    MouseButton::Left => {
                        selected.player1 = Some(character_btn.0);
                        info!("Player1 chose: {:?}", selected.player1);
                    }
                    MouseButton::Right => {
                        selected.player2 = Some(character_btn.0);
                        info!("Player2 chose: {:?}", selected.player2);
                    }
                    _ => {}
                }
            }
        }
    }
}



fn button_visual_feedback(
    mut query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut color) in query.iter_mut() {
        *color = match *interaction {
            Interaction::Pressed => PRESSED_BUTTON.into(),
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON.into(),
        };
    }
}

fn exit_selection(
    //selected: Option<Res<SelectedCharacters>>,
    //mut next_state: ResMut<NextState<GameStates>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Enter)
    {
        //next_state.set(GameStates::Next);
    }
}

fn cleanup_selection_ui(
    mut commands: Commands,
    query: Query<Entity, With<SelectionUI>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}