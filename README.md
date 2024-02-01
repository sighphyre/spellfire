# Spellfire

### What's this?

Spellfire is me screwing around with AI, if you want something useful you should look elsewhere.

If you want something sane you should definitely look elsewhere. I'm planning on doing some terrible things with the Python type system here.

Specifically, I'd like to start experimenting with ideas around how to architect AI based applications. I personally feel this is requires a significantly different structure from traditional applications due to the probabilistic nature of AI and that we're kinda still figuring this out. Exciting times!

Things I'd kinda like to poke at:

- Structured ways of forming prompts based on type systems, Python is goopy enough for this right now but ideally Rust long term
- Equivalent ways of destructuring prompts into proper objects, what invalid responses mean, partial destructing or patterns around validation
- Layering prompts and dealing with equivalent layered failures


### Development

You'll need to setup a venv and install the requirements with `pip install -r requirements.txt`s

Tests are a bit janky right now but you can run them with `pytest test_**.py -vv`