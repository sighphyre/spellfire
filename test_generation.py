from generation import AiAbstractFactory
from dataclasses import dataclass
from typing import Optional


@dataclass
class MockBadger:
    name: str
    likes_honey: Optional[bool]

    @staticmethod
    def prompt_descriptor(*args, **kwargs):
        return f"""
            Make a badger, but with a top-hat.
        """


def make_mock_completer(input):
    def completer(*args):
        return input

    return completer


def test_basic_generation():
    completer = make_mock_completer(
        """
        {
            "name": "Samuel, bringer of vengeance, destroyer of hope, and eater of worlds",
            "likes_honey": true
        }
        """
    )
    factory = AiAbstractFactory(completer=completer)
    character = factory.request_world_object(MockBadger)
    assert (
        character.name
        == "Samuel, bringer of vengeance, destroyer of hope, and eater of worlds"
    )


def test_generator_refuses_to_create_data_class_that_cannot_be_recreated():
    @dataclass
    class NoPromptMethodOnThis:
        is_silly: bool

    completer = make_mock_completer(
        """
        {
            "is_silly": true
        }
        """
    )
    factory = AiAbstractFactory(completer=completer)
    try:
        factory.request_world_object(NoPromptMethodOnThis)
        raise Exception("Object was created that has no prompt method")
    except TypeError:
        pass


def test_generator_rejects_inputs_that_are_missing_required_fields():
    completer = make_mock_completer(
        """
        {
            "has_a_silly_hat": true
        }
        """
    )
    factory = AiAbstractFactory(completer=completer)
    try:
        factory.request_world_object(MockBadger)
        raise Exception("Object was created that was missing a required field")
    except ValueError:
        pass


def test_generator_defaults_missing_optionals_to_none():
    completer = make_mock_completer(
        """
        {
            "name": "Samuel, bringer of vengeance, destroyer of hope, and eater of worlds"
        }
        """
    )
    factory = AiAbstractFactory(completer=completer)
    character = factory.request_world_object(MockBadger)
    assert character.likes_honey == None
