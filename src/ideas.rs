use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::PlayerAction;

pub struct IdeaPlugin;

impl Plugin for IdeaPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerIdeas::with_ideas(vec![Idea::Cube, Idea::Spring]))
            .add_system(cycle_ideas)
            .add_system(load_current_idea)
            .add_system(unload_ideas);
    }
}

#[derive(Resource, Default)]
pub struct PlayerIdeas {
    pub ideas: Vec<Idea>,
    pub available_ideas: Vec<Idea>,
    pub loaded_ideas: Vec<Idea>,
    pub current_index: usize,
}

impl PlayerIdeas {
    pub fn with_ideas(ideas: Vec<Idea>) -> Self {
        PlayerIdeas {
            available_ideas: ideas.clone(),
            ideas,
            loaded_ideas: Vec::new(),
            current_index: 0,
        }
    }

    pub fn ideas_available(&self) -> bool {
        self.available_ideas.len() > 0
    }

    pub fn ideas_loaded(&self) -> bool {
        self.loaded_ideas.len() > 0
    }

    pub fn scroll_forward(&mut self) {
        let new_index = self.current_index + 1;
        if new_index > self.available_ideas.len() - 1 {
            self.current_index = 0;
        } else {
            self.current_index = new_index;
        }
    }

    pub fn scroll_backward(&mut self) {
        if self.current_index == 0 {
            self.current_index = self.available_ideas.len() - 1;
        } else {
            self.current_index -= 1;
        }
    }

    pub fn recall_all_ideas(&mut self) {
        self.available_ideas = self.ideas.clone();
    }

    pub fn recall_ideas(&mut self, ideas_to_recall: Vec<Idea>) {
        for idea in ideas_to_recall {
            self.available_ideas.push(idea);
        }
    }

    pub fn load_idea(&mut self) {
        if self.ideas_available() {
            let idea_to_load = self.available_ideas.remove(self.current_index);
            self.loaded_ideas.push(idea_to_load);
            if self.available_ideas.len() > 0 {
                self.scroll_backward();
            } else {
                self.current_index = 0;
            }
        }
    }

    pub fn unload_ideas(&mut self) {
        while self.ideas_loaded() {
            self.available_ideas.push(self.loaded_ideas.pop().unwrap());
        }
    }

    pub fn spend_ideas(&mut self, ideas_to_spend: Vec<Idea>) {
        for idea in ideas_to_spend {
            let index = self
                .available_ideas
                .iter()
                .position(|x| *x == idea)
                .unwrap();
            self.available_ideas.remove(index);
        }
    }

    pub fn get_idea(&mut self, idea: Idea) {
        self.ideas.push(idea);
        self.available_ideas.push(idea);
    }

    pub fn get_current_idea_tag(&self) -> Option<String> {
        if self.available_ideas.len() > 0 {
            Some(self.available_ideas[self.current_index].to_string())
        } else {
            None
        }
    }

    pub fn get_loaded_idea_at(&self, index: usize) -> Option<String> {
        if self.loaded_ideas.len() > index {
            Some(self.loaded_ideas[index].to_string())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum Idea {
    Cube,
    Spring,
    Wheel,
    Rope,
}

impl std::fmt::Display for Idea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Idea::*;
        match *self {
            Cube => write!(f, "Cube"),
            Spring => write!(f, "Spring"),
            Wheel => write!(f, "Wheel"),
            Rope => write!(f, "Rope"),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum CreationType {
    Crate,
    Launcher,
    PogoStick,
}

#[derive(Component)]
pub struct Creation;

impl CreationType {
    pub fn from_ideas(mut ideas: Vec<&Idea>) -> Option<Self> {
        ideas.sort_by(|a, b| a.partial_cmp(b).unwrap());
        ideas.dedup();
        let idea_count = ideas.len();
        if idea_count > 3 {
            return None;
        }
        let mut sorted_iter = ideas.iter();

        match idea_count {
            2 => match sorted_iter.next().unwrap() {
                Idea::Cube => match sorted_iter.next().unwrap() {
                    Idea::Spring => Some(CreationType::Launcher),
                    _ => None,
                },
                _ => None,
            },

            1 => match sorted_iter.next().unwrap() {
                Idea::Cube => Some(CreationType::Crate),
                Idea::Spring => Some(CreationType::PogoStick),
                _ => None,
            },
            _ => None,
        }
    }
}

pub fn cycle_ideas(
    mut player_ideas: ResMut<PlayerIdeas>,
    query: Query<&ActionState<PlayerAction>>,
) {
    for action in &query {
        if action.just_pressed(PlayerAction::CycleIdeasForward) {
            player_ideas.scroll_forward();
        }
        if action.just_pressed(PlayerAction::CycleIdeasBackward) {
            player_ideas.scroll_backward();
        }
    }
}

pub fn unload_ideas(
    mut player_ideas: ResMut<PlayerIdeas>,
    query: Query<&ActionState<PlayerAction>>,
) {
    for action in &query {
        if action.just_pressed(PlayerAction::UnloadIdeas) {
            player_ideas.unload_ideas();
        }
    }
}

pub fn load_current_idea(
    mut player_ideas: ResMut<PlayerIdeas>,
    query: Query<&ActionState<PlayerAction>>,
) {
    for action in &query {
        if action.just_pressed(PlayerAction::LoadIdea) {
            player_ideas.load_idea();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn creation_single_idea() {
        let new_crate = CreationType::from_ideas(vec![&Idea::Cube]).unwrap();
        assert_eq!(new_crate, CreationType::Crate);
    }

    #[test]
    fn creation_too_many_ideas() {
        use Idea::*;
        let new_creation = CreationType::from_ideas(vec![&Cube, &Spring, &Rope, &Wheel]);
        assert_eq!(new_creation, None);
    }

    #[test]
    fn creation_dedupe_ideas() {
        use Idea::*;
        let trampoline_box =
            CreationType::from_ideas(vec![&Cube, &Spring, &Cube, &Spring, &Cube, &Spring]).unwrap();
        assert_eq!(trampoline_box, CreationType::Launcher);
    }

    #[test]
    fn player_ideas_recall_all_ideas() {
        use Idea::*;
        let mut player_ideas = PlayerIdeas::with_ideas(vec![Cube, Spring, Rope]);
        player_ideas.spend_ideas(vec![Cube, Rope]);
        assert_eq!(player_ideas.available_ideas, vec![Spring]);
        player_ideas.recall_all_ideas();
        assert_eq!(player_ideas.available_ideas, vec![Cube, Spring, Rope]);
    }

    #[test]
    fn player_ideas_recall_specific_ideas() {
        use Idea::*;
        let mut player_ideas = PlayerIdeas::with_ideas(vec![Cube, Spring, Rope]);
        player_ideas.spend_ideas(vec![Cube, Rope, Spring]);
        assert_eq!(player_ideas.available_ideas, Vec::new());
        player_ideas.recall_ideas(vec![Rope, Spring]);
        assert_eq!(player_ideas.available_ideas, vec![Rope, Spring]);
    }
}
