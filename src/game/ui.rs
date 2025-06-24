use bevy::window::PrimaryWindow;

use self::level::Levels;

use super::{
    cursor::click_detection, level::{BreadCount, CurrentLevelIndex, TotalBreadCount}, *
};
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup,show_title_and_name)
        
        .add_systems(OnEnter(GameStates::Next), show_level_title)
        .add_systems(OnEnter(GameStates::Next), show_stuffed_ducks_count)
        .add_event::<Won>()
        .add_systems(
            Update,
            (
                won.run_if(in_state(GameStates::Next)),
                update_level_title.run_if(in_state(GameStates::Next)),
                show_hints.run_if(in_state(GameStates::Next)),
                next_level_button_interaction.after(click_detection)

                // next_level_button_interaction should execute after click_detection
                // It fixes the bug when click the next level button and a duck simsimultaneously
                // If not doing so, click_detection will try to insert Player bundle to an invalid entity, causes the game to crash
                //next_level_button_interaction.after(click_detection),
            ),
        )
        .add_systems(Update, update_stuffed_ducks_count.run_if(in_state(GameStates::Next)));
    }
}

fn show_title_and_name(mut commands: Commands, asset_server: Res<AssetServer>) {
    // game title
    commands.spawn(
        TextBundle::from_section(
            "BATTLE on ICE!",
            TextStyle {
                font: asset_server.load("fonts/NotJamChunky8.ttf"),
                font_size: 30.0,
                color: MY_ORANGE,
            },
        )
        .with_text_justify(JustifyText::Right)
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        }),
    );

    // author name
    commands.spawn(
        TextBundle::from_section(
            "Have FUN with your friend!",
            TextStyle {
                font: asset_server.load("fonts/NotJamChunky8.ttf"),
                font_size: 20.0,
                ..default()
            },
        )
        .with_text_justify(JustifyText::Right)
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(50.0),
            right: Val::Px(10.0),
            ..default()
        }),
    );
}

#[derive(Component)]
pub struct StuffedDucksCount;

fn show_stuffed_ducks_count(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    bread_count: Res<BreadCount>,
    total_bread_count: Res<TotalBreadCount>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.get_single().unwrap();
    commands.spawn((
        TextBundle::from_section(
            format!(
                "{}/{}",
                total_bread_count.0 - bread_count.0,
                total_bread_count.0
            ),
            TextStyle {
                font: asset_server.load("fonts/NotJamChunky8.ttf"),
                font_size: 30.0,
                ..default()
            },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(window.width() / 2.0 - 45.0),
            ..default()
        }),
        StuffedDucksCount,
    ));
}

fn update_stuffed_ducks_count(
    bread_count: Res<BreadCount>,
    total_bread_count: Res<TotalBreadCount>,
    mut stuffed_ducks_count: Query<(&mut Text, &mut Style), With<StuffedDucksCount>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_query.get_single()else{return;};
    for (mut text, mut style) in stuffed_ducks_count.iter_mut() {
        text.sections[0].value = format!(
            "{}/{}",
            total_bread_count.0 - bread_count.0,
            total_bread_count.0
        );
        // TODO: Find a better way to do UI alighment after resizing the window
        // window resize event should also change the "RESIZE"
        style.right = Val::Px(window.width() / 2.0 - 45.0);
    }
}

#[derive(Component)]
pub struct LevelTitle;

fn show_level_title(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    level_index: Res<CurrentLevelIndex>,
) {
    commands.spawn((
        TextBundle::from_section(
            format!("Level{}", level_index.0),
            TextStyle {
                font: asset_server.load("fonts/NotJamChunky8.ttf"),
                font_size: 30.0,
                ..default()
            },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        LevelTitle,
    ));
}

// HINTS:
// Click to choose the duck
// WASD to move
// R to reset
// Z to undo
#[derive(Component)]
pub struct GameHints;

fn show_hints(mut commands: Commands, asset_server: Res<AssetServer>) {
    let text_style_important = TextStyle {
        font: asset_server.load("fonts/NotJamChunky8.ttf"),
        font_size: 20.0,
        color: MY_ORANGE,
    };
    let text_style_normal = TextStyle {
        font: asset_server.load("fonts/NotJamChunky8.ttf"),
        font_size: 20.0,
        ..default()
    };
    commands.spawn((TextBundle::from_sections([
        TextSection::new("Click ", text_style_important.clone()),
        TextSection::new("to choose your sprite\n", text_style_normal.clone()),
        TextSection::new("WASD or Arrow ", text_style_important.clone()),
        TextSection::new("to move\n", text_style_normal.clone()),
        TextSection::new("R ", text_style_important.clone()),
        TextSection::new("to reset\n", text_style_normal.clone()),
        //TextSection::new("Z ", text_style_important.clone()),
        //TextSection::new("to undo\n", text_style_normal.clone()),
        TextSection::new("[ ] ", text_style_important.clone()),
        TextSection::new("to skip levels\n\n", text_style_normal.clone()),
    ])
    .with_text_justify(JustifyText::Right)
    .with_style(Style {
        position_type: PositionType::Absolute,
        top: Val::Px(10.0),
        right: Val::Px(10.0),
        ..default()
    }),
    GameHints,
));
}

fn update_level_title(
    mut commands: Commands,
    level_index: Res<CurrentLevelIndex>,
    mut level_title: Query<&mut Text, With<LevelTitle>>,
    ui_query: Query<Entity, With<MutUI>>,
) {
    if level_index.is_changed() {
        for mut text in level_title.iter_mut() {
            text.sections[0].value = format!("Level{}", level_index.0);
        }
        //info!("despawn!");
        // Despawn ui elements
        for entity in ui_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

const NORMAL_BUTTON: Color = MY_ORANGE;
const HOVERED_BUTTON: Color =
    Color::srgb(222.0 / 255.0 + 0.1, 112.0 / 255.0 + 0.1, 40.0 / 255.0 + 0.1);
const PRESSED_BUTTON: Color = Color::srgb(0.75, 0.75, 0.75);

#[derive(Event, Default)]
pub struct Won;

#[derive(Component)]
pub struct MutUI;

#[derive(Component)]
pub struct NextLevelButton;

fn won(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut events: EventReader<Won>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    level_index: Res<CurrentLevelIndex>,
    levels: Res<Levels>,
) {
    for _ in events.read() {
        let window = window_query.get_single().unwrap();
        commands.spawn((
            TextBundle::from_section(
                "Yummy!",
                TextStyle {
                    font: asset_server.load("fonts/NotJamChunky8.ttf"),
                    font_size: 40.0,
                    color: MY_ORANGE,
                },
            )
            .with_text_justify(JustifyText::Center)
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(40.0),
                right: Val::Px(window.width() / 2.0 - 140.0),
                ..default()
            }),
            MutUI,
        ));

        let mut button_text = "Next Level";
        if level_index.0 == levels.levels.len() {
            button_text = "You WIN!";
        }

        // Show next level button
        commands
            .spawn((NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },))
            .insert(MutUI)
            .with_children(|parent| {
                parent
                    .spawn(ButtonBundle {
                        style: Style {
                            width: Val::Px(200.0),
                            height: Val::Px(80.0),
                            border: UiRect::all(Val::Px(5.0)),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    })
                    .insert(MutUI)
                    .insert(NextLevelButton)
                    .with_children(|parent| {
                        parent
                            .spawn(TextBundle::from_section(
                                button_text,
                                TextStyle {
                                    font: asset_server.load("fonts/NotJamChunky8.ttf"),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            ))
                            .insert(MutUI);
                    });
            });
    }
}

use renet::RenetClient;
use crate::networking::ClientMessage;
fn next_level_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, (With<Button>, With<NextLevelButton>)),
    >,
    mut level_index: ResMut<CurrentLevelIndex>,
    levels: Res<level::Levels>,
    mut client: Option<ResMut<RenetClient>>,

) {
    // Handle invalid level index
    if level::load_level(level_index.0, levels).is_err() {
        info!("Invalid level index");
        level_index.0 -= 1;
        return;
    }

    for (interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::WHITE;
                // If we're the client, send next level request
                if let Some(client) = &mut client {
                    let message = bincode::serialize(&ClientMessage::NextLevelRequest).unwrap();
                    client.send_message(0, message);
                    info!("next level request!");
                } 

            }

            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = MY_BROWN;
            }
        }
    }
}
