# Project setup

## Project creation
The first thing we have to do is to create the tuber project for our game.

To do so, we first need to build a new rust binary crate.

```bash 
cargo new --bin escape-orcs-2
```

## Dependencies
In order to use tuber in our project, we need to add it as a dependency of our crate.

In the ``Cargo.toml`` file of the generated crate, add the tuber crate in the dependencies section, as shown below.

```toml
[package]
name = "escape-orcs-2"
version = "0.1.0"
edition = "2021"
 
[dependencies]
tuber = "0.1"
```

## Initialization
Now, we need to initialize a new instance of the tuber engine.

In the ``main.rs`` of your crate, add the following code

```rust,noplayground
use tuber::engine::{Engine, EngineSettings, TuberRunner, Result};
use tuber::WinitTuberRunner;

fn main() -> Result<()> {
    let engine = Engine::new(EngineSettings {
        application_title: Some("Escape Orcs 2".into()),
        initial_state: None
    });

    WinitTuberRunner.run(engine)
}
```

The ``Engine`` struct represents the engine in itself, it's the entry point of your tuber application and holds all the 
engine state.

We pass to its constructor a ``EngineSettings`` struct, this is a set of settings that the engine will use. Here we are 
setting the title of the application to be ``Escape Orcs 2``, that will be the title of the game's window.

``WinitTuberRunner`` allows to run the engine using [winit](https://github.com/rust-windowing/winit) as the windowing 
backend of the engine. As of now this is the only windowing backend supported by tuber.

Let's try to run the game now !
```bash
cargo run
```

Oh, wait... It crashed!

This is because the engine starts without having any game state to run.
We need to create a game state.

## Game state creation

A game has multiple states, it could be showing the main menu, the settings menu, showing the world map, playing the 
game. When the engine starts, it must have a game state to run, otherwise, there is no game.

In order to create a game state we have to create a struct implementing the ``State`` trait.

Let's do so by creating an empty ``GameState`` struct implementing the ``State`` trait after our main function.
```rust,noplayground
use tuber::engine::state::State;

// ...

struct GameState;
impl State for GameState {}
```

Now we need to define that state as the initial state of the engine
```rust,noplayground
fn main() -> Result<()> {
    let engine = Engine::new(EngineSettings {
        application_title: Some("Escape Orcs 2".into()),
        initial_state: Some(Box::new(GameState)) // Sets the initial state as GameState
    });

    WinitTuberRunner.run(engine)
}
```

Let's run the game again...

It didn't crash! Instead it shows a black window with the title we defined.
