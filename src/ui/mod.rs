use bevy::prelude::*;

use crate::PlayerIdeas;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_hud)
            .add_system(handle_selected_idea_text)
            .add_system(handle_loaded_ideas_text);
    }
}

#[derive(Component)]
pub struct CurrentIdeaText;

#[derive(Component)]
pub struct LoadedIdeasText;

#[derive(Component)]
pub struct LeftHud;

fn handle_selected_idea_text(
    player_ideas: Res<PlayerIdeas>,
    mut query: Query<&mut Text, With<CurrentIdeaText>>,
) {
    if player_ideas.is_changed() {
        for mut text in &mut query {
            if let Some(idea_tag) = player_ideas.get_current_idea_tag() {
                text.sections[1].value = idea_tag;
            } else {
                text.sections[1].value = "".to_string();
            }
        }
    }
}

fn handle_loaded_ideas_text(
    player_ideas: Res<PlayerIdeas>,
    mut query: Query<&mut Text, With<LoadedIdeasText>>,
) {
    if player_ideas.is_changed() {
        for mut text in &mut query {
            if let Some(idea_tag) = player_ideas.get_loaded_idea_at(2) {
                text.sections[3].value = idea_tag;
            }
            if let Some(idea_tag) = player_ideas.get_loaded_idea_at(1) {
                text.sections[2].value = idea_tag;
            }
            if let Some(idea_tag) = player_ideas.get_loaded_idea_at(0) {
                text.sections[1].value = idea_tag;
            }
        }
    }
}

fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("FiraSans-Bold.ttf");
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::width(Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::width(Val::Percent(20.0)),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    ..default()
                })
                .insert(LeftHud)
                .with_children(|parent_2| {
                    parent_2
                        .spawn(TextBundle::from_sections([
                            TextSection::new(
                                "Ideas: ",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 30.0,
                                    color: Color::WHITE,
                                },
                            ),
                            TextSection::new(
                                "Empty",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 30.0,
                                    color: Color::TEAL,
                                },
                            ),
                        ]))
                        .insert(CurrentIdeaText);
                    parent_2
                        .spawn(TextBundle::from_sections([
                            TextSection::new(
                                "Loaded Ideas: ",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 30.0,
                                    color: Color::WHITE,
                                },
                            ),
                            TextSection::new(
                                " ",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 30.0,
                                    color: Color::RED,
                                },
                            ),
                            TextSection::new(
                                " ",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 30.0,
                                    color: Color::ORANGE,
                                },
                            ),
                            TextSection::new(
                                " ",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 30.0,
                                    color: Color::YELLOW,
                                },
                            ),
                        ]))
                        .insert(LoadedIdeasText);
                });
        });
}
