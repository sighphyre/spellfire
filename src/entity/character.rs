use serde::{Deserialize, Serialize};

use super::SelfDescribe;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Character {
    name: String,
    description: String,
    gender: String,
    race: String,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            name: "Analia".into(),
            description: "The elf queen of Eldulia is a regal and graceful leader, revered by her people for her wisdom and kindness. With long, flowing silver hair and sparkling emerald eyes, she exudes an ethereal beauty that belies her immense power and strength. Clad in ornate, intricately woven robes that shimmer with the colors of the forest, she is a symbol of the natural world and its enduring magic. Her presence commands respect, and her words hold great weight among her subjects and allies. Despite her elegance, the elf queen is a formidable warrior, skilled in both magic and swordplay, and fiercely protective of her homeland and its inhabitants. She is a beacon of hope and guidance for the elves of Eldulia, guiding them with compassion and unwavering resolve in the face of adversity.".into(),
            gender: "Female".into(),
            race: "Elf".into(),
        }
    }
}

impl SelfDescribe for Character {
    type Input = String;

    fn describe(&self, input: &Self::Input) -> String {
        let example = serde_json::to_string(&self).unwrap();
        format!(
            "Generate a description for a character for a game.\n\
            This should be {input}.\n\
            Only return answers in the following format:\n\
            {example}"
        )
    }
}
