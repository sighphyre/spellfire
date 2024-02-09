use serde::{Deserialize, Serialize};

use super::{character::Character, location::Location, SelfDescribe};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Encounter {
    name: String,
    description: String,
    location: String,
    characters: Vec<Character>,
}

impl Default for Encounter {
    fn default() -> Self {
        Self {
            name: "The Elf Queen".into(),
            description: "The elf queen of Eldulia is a regal and graceful leader, revered by her people for her wisdom and kindness. With long, flowing silver hair and sparkling emerald eyes, she exudes an ethereal beauty that belies her immense power and strength. Clad in ornate, intricately woven robes that shimmer with the colors of the forest, she is a symbol of the natural world and its enduring magic. Her presence commands respect, and her words hold great weight among her subjects and allies. Despite her elegance, the elf queen is a formidable warrior, skilled in both magic and swordplay, and fiercely protective of her homeland and its inhabitants. She is a beacon of hope and guidance for the elves of Eldulia, guiding them with compassion and unwavering resolve in the face of adversity.".into(),
            location: "Eldulia".into(),
            characters: vec![Character::default()],
        }
    }
}

impl SelfDescribe for Encounter {
    type Input = (Location, String);

    fn describe(&self, input: &Self::Input) -> String {
        let (location, _description) = input.clone();

        let location_name = location.name;
        let location_description = location.description;

        let example = serde_json::to_string(&self).unwrap();

        format!(
            "Generate a description for an encounter for a game. \n\
            This takes place in {location_name},\n\
            which is described by {location_description}.\n\
            Only return answers in the following format:\n
            {example}"
        )
    }
}
