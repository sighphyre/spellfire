from typing import Optional
from dataclasses import dataclass, asdict
import json
from openai import OpenAI


def from_dict(cls, raw_data: dict):
    if not (
        hasattr(cls, "prompt_descriptor")
        and callable(getattr(cls, "prompt_descriptor"))
    ):
        raise TypeError("The class does not have a prompt_descriptor method")

    ## We're gonna cheat a little here and use the dataclass's annotations
    ## to decide on our own validation. Super hacky.
    ##
    ## Also pretty sure I definitely wanna do this
    annotations = cls.__annotations__
    required_keys = [
        key for key in annotations.keys() if not annotations[key].__module__ == "typing"
    ]

    if not all(key in raw_data.keys() for key in required_keys):
        raise ValueError(
            f"One or more of the following keys are missing: {required_keys}"
        )

    optional_keys = [
        key for key in annotations.keys() if annotations[key].__module__ == "typing"
    ]
    combined_data = {**raw_data, **{key: None for key in optional_keys}}

    return cls(**combined_data)


@dataclass
class Character:
    name: str
    description: str
    gender: str

    @classmethod
    def prompt_descriptor(cls, description: str):
        EXAMPLE = from_dict(
            cls=cls,
            raw_data={
                "name": "Analia",
                "description": "The elf queen of Eldulia is a regal and graceful leader, revered by her people for her wisdom and kindness. With long, flowing silver hair and sparkling emerald eyes, she exudes an ethereal beauty that belies her immense power and strength. Clad in ornate, intricately woven robes that shimmer with the colors of the forest, she is a symbol of the natural world and its enduring magic. Her presence commands respect, and her words hold great weight among her subjects and allies. Despite her elegance, the elf queen is a formidable warrior, skilled in both magic and swordplay, and fiercely protective of her homeland and its inhabitants. She is a beacon of hope and guidance for the elves of Eldulia, guiding them with compassion and unwavering resolve in the face of adversity.",
                "gender": "Female",
            },
        )

        return f"""

        Generate a description for a character for a game. This should be {description}.

        Include a character sheet in JSON format. Example:

        {json.dumps(asdict(EXAMPLE))}

        """


def create_completer(client):
    def complete(cls, params):
        chat_completion = client.chat.completions.create(
            messages=[
                {
                    "role": "user",
                    "content": cls.prompt_descriptor(*params),
                }
            ],
            model="gpt-3.5-turbo-1106",
        )

        return chat_completion.choices[0].message.content

    return complete


class AiAbstractFactory:
    def __init__(self, completer):
        self.completer = completer

    def request_world_object(self, cls, params=None):
        chat_completion = self.completer(cls, params)

        message_dict = json.loads(chat_completion)

        return from_dict(cls, message_dict)


def default_factory():
    return create_completer(
        client=OpenAI(api_key=open(".openapikey").read().strip()),
    )
