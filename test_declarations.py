from declarations import Character, Location, Encounter
import inspect
from dataclasses import is_dataclass


## Nothing but a sniff test to see that our declarations are valid
def test_declarations_can_produce_prompts():
    stringy_class = [Location, Character]

    for cls in stringy_class:
        prompt = cls.prompt_descriptor("a dude with a top hat")
        assert isinstance(prompt, str)

def test_encounters_can_be_built_off_locations():
    location = Location(name="The Forest of Eldulia", description="The Forest of Eldulia is a sprawling and ancient woodland, teeming with life and mystery. Towering trees loom overhead, their branches interlocking to create a canopy that filters the sunlight into a dappled, ethereal glow. The forest floor is carpeted with lush mosses and ferns, concealing hidden glades and meandering streams that wind their way through the landscape.The air is alive with the rustle of leaves and the melodic calls of birds and woodland creatures. But beyond the tranquil facade, there is an undercurrent of magic and danger. Ancient ruins and forgotten relics lie buried beneath the forest's undergrowth, hinting at a forgotten civilization that once thrived in these woods.Adventurers and explorers are drawn to The Forest of Eldulia, eager to uncover its secrets and uncover the truth of its mystical inhabitants. But they must tread carefully, for the forest is shrouded in enchantments and guarded by creatures both wondrous and fearsome. The Forest of Eldulia is a place of beauty, enchantment, and peril, where the line between reality and myth begins to blur.")

    prompt = Encounter.prompt_descriptor(location)
    assert isinstance(prompt, str)
