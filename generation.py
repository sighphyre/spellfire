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


def create_completer(client):
    def complete(cls, params):

        prompt = cls.prompt_descriptor(*params)
        print("This is mah Prompt: ", prompt)

        chat_completion = client.chat.completions.create(
            messages=[
                {
                    "role": "user",
                    "content": prompt,
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

        print("completion", chat_completion)
        message_dict = json.loads(chat_completion)

        return from_dict(cls, message_dict)


def default_factory():
    return AiAbstractFactory(
        create_completer(
            client=OpenAI(api_key=open(".openapikey").read().strip()),
        )
    )
