# DuoSnake 🐍🐍

DuoSnake is a competitive, two-player snake game built with the [Bevy](https://bevyengine.org/) engine (v0.14). It introduces survival and combat mechanics to the classic snake formula.

## Features

- **Two-Player Co-op / Versus**: Play with a friend locally. Player 1 and Player 2 share the arena.
- **HP & Respawn System**: Unlike traditional snake where one hit means game over, DuoSnake gives players health points (HP). Dying triggers a respawn timer instead of ending the game immediately.
- **Dynamic Environment**: Collect food to grow, but watch out for randomly generating timed traps!
- **Settings & Pause UI**: In-game menus to manage settings (like initial HP) and pause the game at any time.

## Project Structure

This project follows Bevy's ECS (Entity Component System) architecture:

- `src/main.rs`: Entry point. Manages game states (`GameState::Playing`, `Paused`, `Settings`) and system initialization.
- `src/components.rs`: Defines core data structures like `Player`, `Position`, `SnakeHead`, `Trap`, `PlayerHP`, etc.
- `src/constants.rs`: Hard-coded game configurations (grid size, colors, etc.).
- `src/systems/`: The game logic separated by domain:
  - `snake.rs`: Movement, input, eating, growth, and death/respawn logic.
  - `environment.rs`: Spawning mechanics for food and traps.
  - `ui.rs`: Handlers for menus, score/HP display, and interactive buttons.
  - `render.rs`: Transforms logical grid coordinates into visual screen positions and scales.

## Running the Game

Make sure you have Rust installed. Then, from the project root, run:

```bash
cargo run --release
```

Enjoy the chaotic arena!
