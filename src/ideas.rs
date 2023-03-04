use bevy::prelude::*;

pub struct PlayerIdeas {
    pub ideas: Vec<Idea>,
    pub available_ideas: Vec<Idea>,
}

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum Idea {
    Cube,
    Spring,
    Wheel,
    Rope,
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn single_idea() {
        let new_crate = CreationType::from_ideas(vec![&Idea::Cube]).unwrap();
        assert_eq!(new_crate, CreationType::Crate);
    }

    #[test]
    fn too_many_ideas() {
        use Idea::*;
        let new_creation = CreationType::from_ideas(vec![&Cube, &Spring, &Rope, &Wheel]);
        assert_eq!(new_creation, None);
    }

    #[test]
    fn dedupe_ideas() {
        use Idea::*;
        let trampoline_box =
            CreationType::from_ideas(vec![&Cube, &Spring, &Cube, &Spring, &Cube, &Spring]).unwrap();
        assert_eq!(trampoline_box, CreationType::Launcher);
    }
}
