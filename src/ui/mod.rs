use bevy::prelude::*;

use crate::PlayerIdeas;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_hud)
            .add_system(handle_current_idea_text);
    }
}

#[derive(Component)]
pub struct CurrentIdeaText;

fn handle_current_idea_text(
    player_ideas: Res<PlayerIdeas>,
    mut query: Query<&mut Text, With<CurrentIdeaText>>,
) {
    if player_ideas.is_changed() {
        for mut text in &mut query {
            text.sections[1].value = player_ideas.get_current_idea_tag();
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
                        ..default()
                    },
                    ..default()
                })
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
                });
        });
}
