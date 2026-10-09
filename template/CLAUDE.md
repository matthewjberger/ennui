@../RULES.md
@../INTERFACE.md

# Template

This is an ennui app in its own repository. Its justfile names the ennui clone (`engine :=`); `Cargo.toml` builds the ennui crates from that clone by path, and the editor is built from the same clone.

- Start work on the interface with the `/editor` skill: it is the guide for screens, text tables and themes, and for driving the editor. The skills in `.claude/skills/` and the editor hook in `.claude/settings.json` come from the ennui clone through `just sync`; do not edit them here.
- `just run` runs the app, `just edit [screen]` opens the editor on `project/`, `just trace` records where frame time goes, `just check` runs clippy and a play build, `just export` builds the app for release beside a copy of its `project/` folder in `target/shipped`, `just lint` and `just fmt` check and format the code, and `just sync` copies the ennui workspace settings after a change to them.
- `src/` holds start-up, the app state and the screens it opens. UI screens and text tables are scenes in `project/scenes/` (`ui/` and `text/`); `project/settings.scene` sets the theme. Every asset the app uses is committed under `assets/` and named as an `assets/...` path.
