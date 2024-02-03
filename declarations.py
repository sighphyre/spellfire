from dataclasses import dataclass
import json
from enum import Enum


class Nature(Enum):
    BANISHED = 1
    WILD = 2
    CIVILIZED = 3
    NOBLE = 4


class Race(Enum):
    ELF = 1
    ORC = 2
    HUMAN = 3
    DWARF = 4
    FAIRY = 5
    OGRE = 6
    HALFLING = 7
    CENTAUR = 8


class Alignment(Enum):
    GOOD = 1
    NEUTRAL = 2
    EVIL = 3


def format_example(example: dict):
    return f"""

    Only return answers in the following format:

    {json.dumps(example)}

    """


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
            "race": "Elf",
        }

        return f"""

        Generate a description for a character for a game. This should be {description}.

        {format_example(example)}
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

        {format_example(example)}
        """


@dataclass
class CharacterClass:
    kind: str
    level: int
    attack: int
    defense: int
    description: str
    int: int
    str: int
    con: int

    @staticmethod
    def prompt_descriptor(description: str):
        example = {
            "kind": "Fighter",
            "level": 10,
            "attack": 50,
            "defense": 40,
            "int": 30,
            "str": 60,
            "con": 50,
            "description": "The Fighter is a versatile warrior skilled in combat, excelling in both offensive and defensive strategies. Equipped with a range of weapons and armor, Fighters can adapt to any battlefield situation. They have high health and strength, making them formidable frontline combatants. Fighters can specialize in different weapons and tactics, allowing for personalized combat styles. Their resilience and combat prowess make them essential in any team formation.",
        }

        return f"""

        Generate a description for a character class. This should be {description}.

        {format_example(example)}"""


@dataclass
class Spell:
    name: str
    mana_cost: int
    aoe: bool
    effect: str
    self_cast: bool
    hostile: bool
    description: str

    @staticmethod
    def prompt_descriptor(description: str):
        example = {
            "mana_cost": 20,
            "aoe": True,
            "name": "Fireball",
            "effect": "damage target",
            "self_cast": False,
            "hostile": True,
            "description": "The Fireball spell is a powerful offensive spell that conjures a ball of fire and hurls it at the target. It deals massive damage on impact and can ignite the target, causing additional damage over time. The Fireball spell is a staple in the arsenal of any mage, capable of decimating groups of enemies and turning the tide of battle in an instant. Its destructive force is unmatched, making it a feared and respected spell among both allies and foes.",
        }

        return f"""

        Generate a description for a spell. This should be {description}.

        The "effect" can only be one of damage, damage over time, heal, heal over time, control target, create creature

        {format_example(example)}
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

        {format_example(example)}
        """


if __name__ == "__main__":
    from generation import default_factory

    factory = default_factory()

    # location = Location(
    #     name="The Forest of Eldulia",
    #     description="The Forest of Eldulia is a sprawling and ancient woodland, teeming with life and mystery. Towering trees loom overhead, their branches interlocking to create a canopy that filters the sunlight into a dappled, ethereal glow. The forest floor is carpeted with lush mosses and ferns, concealing hidden glades and meandering streams that wind their way through the landscape.The air is alive with the rustle of leaves and the melodic calls of birds and woodland creatures. But beyond the tranquil facade, there is an undercurrent of magic and danger. Ancient ruins and forgotten relics lie buried beneath the forest's undergrowth, hinting at a forgotten civilization that once thrived in these woods.Adventurers and explorers are drawn to The Forest of Eldulia, eager to uncover its secrets and uncover the truth of its mystical inhabitants. But they must tread carefully, for the forest is shrouded in enchantments and guarded by creatures both wondrous and fearsome. The Forest of Eldulia is a place of beauty, enchantment, and peril, where the line between reality and myth begins to blur.",
    # )

    # encounter = factory.request_world_object(Character, ["A Halfling, partially Orcish (he has an orc father), he's a thief by trade, he has a very unique scent."])

    char_class = factory.request_world_object(
        Spell,
        ["an evil spell"],
    )

    print(char_class)
