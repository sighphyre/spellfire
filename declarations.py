from dataclasses import dataclass
import json


@dataclass
class Character:
    name: str
    description: str
    gender: str

    @staticmethod
    def prompt_descriptor(description: str):
        example = {
            "name": "Analia",
            "description": "The elf queen of Eldulia is a regal and graceful leader, revered by her people for her wisdom and kindness. With long, flowing silver hair and sparkling emerald eyes, she exudes an ethereal beauty that belies her immense power and strength. Clad in ornate, intricately woven robes that shimmer with the colors of the forest, she is a symbol of the natural world and its enduring magic. Her presence commands respect, and her words hold great weight among her subjects and allies. Despite her elegance, the elf queen is a formidable warrior, skilled in both magic and swordplay, and fiercely protective of her homeland and its inhabitants. She is a beacon of hope and guidance for the elves of Eldulia, guiding them with compassion and unwavering resolve in the face of adversity.",
            "gender": "Female",
        }

        return f"""

        Generate a description for a character for a game. This should be {description}.

        Only return answers in the following format:

        {json.dumps(example)}

        """


@dataclass
class Location:
    name: str
    description: str

    @staticmethod
    def prompt_descriptor(description: str):
        example = {
            "name": "The Forest of Eldulia",
            "description": "The Forest of Eldulia is a sprawling and ancient woodland, teeming with life and mystery. Towering trees loom overhead, their branches interlocking to create a canopy that filters the sunlight into a dappled, ethereal glow. The forest floor is carpeted with lush mosses and ferns, concealing hidden glades and meandering streams that wind their way through the landscape.The air is alive with the rustle of leaves and the melodic calls of birds and woodland creatures. But beyond the tranquil facade, there is an undercurrent of magic and danger. Ancient ruins and forgotten relics lie buried beneath the forest's undergrowth, hinting at a forgotten civilization that once thrived in these woods.Adventurers and explorers are drawn to The Forest of Eldulia, eager to uncover its secrets and uncover the truth of its mystical inhabitants. But they must tread carefully, for the forest is shrouded in enchantments and guarded by creatures both wondrous and fearsome. The Forest of Eldulia is a place of beauty, enchantment, and peril, where the line between reality and myth begins to blur.",
        }

        return f"""

        Generate a description for a location for a game. This should be {description}.

        Only return answers in the following format:

        {json.dumps(example)}

        """


@dataclass
class Encounter:
    description: str

    @staticmethod
    def prompt_descriptor(location: Location):
        example = {
            "description": "As you journey through The Forest of Eldulia, you come across a clearing bathed in a surreal, otherworldly light. The air is thick with the fragrance of wildflowers, and the sounds of the forest seem to hush in reverence. In the center of the clearing, you spot a figure cloaked in shimmering robes, their face shrouded in shadow. As you cautiously approach, the figure raises their head to reveal piercing, glowing eyes that seem to bore into your very soul. The figure introduces themselves as an ancient guardian of the forest, tasked with protecting its secrets and ensuring that only those deemed worthy may pass. They offer you a challenge, a test of your courage and wisdom, to prove yourself as a true friend to the forest. If you succeed, they promise to bestow upon you a powerful artifact that will aid you in your quest. But be warned, for the guardian's challenge is not to be taken lightly. You must navigate a series of trials and puzzles, facing the forest's magical inhabitants and unlocking the secrets of its ancient ruins. As you delve deeper into the guardian's challenge, you begin to sense the very essence of the forest itself, its magic resonating within you and urging you to persevere. But as you near the end of the trial, you realize that there is more at stake than just the artifact. The forest's mystical inhabitants are watching, and they will not take kindly to failure. Will you prove yourself worthy and earn the guardian's favor, or will you fall victim to the forest's enchantments and become just another forgotten relic in its depths? The choice is yours, adventurer.",
        }
        return f"""

        Generate a description for an encounter for a game. This takes place in {location.name}, which is described by {location.description}.

        Only return answers in the following format:

        {json.dumps(example)}`

        """


if __name__ == "__main__":
    from generation import default_factory

    factory = default_factory()

    location = Location(
        name="The Forest of Eldulia",
        description="The Forest of Eldulia is a sprawling and ancient woodland, teeming with life and mystery. Towering trees loom overhead, their branches interlocking to create a canopy that filters the sunlight into a dappled, ethereal glow. The forest floor is carpeted with lush mosses and ferns, concealing hidden glades and meandering streams that wind their way through the landscape.The air is alive with the rustle of leaves and the melodic calls of birds and woodland creatures. But beyond the tranquil facade, there is an undercurrent of magic and danger. Ancient ruins and forgotten relics lie buried beneath the forest's undergrowth, hinting at a forgotten civilization that once thrived in these woods.Adventurers and explorers are drawn to The Forest of Eldulia, eager to uncover its secrets and uncover the truth of its mystical inhabitants. But they must tread carefully, for the forest is shrouded in enchantments and guarded by creatures both wondrous and fearsome. The Forest of Eldulia is a place of beauty, enchantment, and peril, where the line between reality and myth begins to blur.",
    )

    encounter = factory.request_world_object(Encounter, [location])

    print(encounter)
