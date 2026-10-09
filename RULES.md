# ennui rules

These rules apply to all work on ennui: the crates, the editor, the gallery, the template and every app made from it. CLAUDE.md imports this file, and so does the CLAUDE.md of each app that `just new` makes. Claude writes the code; the user decides, looks, uses the apps and says what is wrong.

## The repository

- The crates are grouped as `crates/engine/` (the app core, scenes, text, data binding and screens), `crates/rendering/` (the renderer, the overlay pass and the window), `crates/ui/` (the retained interface and its controls). The editor and the gallery are apps under `apps/`; `template/` is the app that `just new` copies; `tools/ennui` is the command line tool.
- Each repository builds into its own `target/`. Never share a build folder between repositories, and never let a tool start a cargo build in a `target/play` folder by itself: builds with other features break the link of the dynamic library (`ennui_dylib`).
- Every justfile recipe is a one-line call to the `ennui` tool in `tools/ennui` (Rust), so a machine needs only Rust, `just` and git. Put recipe logic in the tool, never in PowerShell, shell or Python scripts.
- After a change to workspace dependencies, profiles, lints, the toolchain or the lock, run `just sync` in `template/` and in the apps being worked on, and commit the result there. The user does not run it; Claude does.
- A crate that one app uses lives under that app, named with its prefix; it moves into `crates/` with an `ennui-` name when a second user needs it. Editor-only code lives in crates under `apps/editor`.
- New apps come from `template/` through `just new <folder>`. Never copy an app by hand from another app.
- ennui is the 2D interface part of a larger engine and stays that: no 3D rendering, physics, terrain or sound. An app that needs those is a game, and belongs on a full engine.

## Design

- Data-oriented: per-entity data in components, shared data in resources, behavior in systems; commands change data, queries read it. Free functions over data; never `impl` methods that take `&mut self` (a builder or a field-filling constructor is fine).
- Systems take `Res`, `ResMut`, `View`, `Peek`, `Mut`, `Glance` and `Later`; never `Rows` or a whole `Resources`: such a system runs alone and blocks parallel batches. Commands and queries take only the data they use. Read the `audit` skill before writing code.
- No one-line functions: inline them at the callers. Exceptions: public API that other crates call, and a lookup repeated at 4 or more sites.
- No global state: no statics, atomics or process-wide switches; settings are resources passed as arguments.
- Every crate manifest has a `[lints]` section with `workspace = true`.
- All Rust: no C or C++ crates (no `-sys`, `cc`, `bindgen`), no `unsafe`, stable mature crates only. The exceptions are what cannot be worked around, and they stay: bindings to OS libraries (the windowing and graphics bindings under `winit` and `wgpu`). Do not add a new one without asking. `nalgebra-glm` stays at 0.20.
- ennui runs on Windows, macOS, Linux and the web (wasm, WebGPU), and support for Android is planned. On the web there are no threads, no file system and no binding arrays: files come from the `Shelf`, rendering runs on the page thread, and pictures bind one at a time. Keep every crate portable: no code that only one desktop platform can run, and platform differences behind `cfg` checks in the crate that needs them.
- Build the best long-term design at the scale of a mature engine's interface layer, but the smallest thing that does what was asked; never overfit the current apps and never add features that nobody asked for. Judge a new feature against ennui as it would be with it.
- Make every major piece work simply first; no churn on minor things.

## Rendering

- There is no fixed frame-rate target; do nothing that is obviously bad for frame time, and measure a new render feature with `just trace` before building it out.
- Render work stays GPU-driven: no CPU estimates or CPU skipping when the GPU can decide through indirect arguments.
- A render feature is its own crate composed into the renderer; never delete one as bloat.
- ennui's names for features are plain words. Search for ennui's words before calling a feature missing.
- A slow first frame is driver pipeline compiles; never use the wgpu pipeline cache.

## Interfaces and the editor

- App UI is authored in scenes (`project/scenes/ui/*.scene`) and built with retained `ennui-ui`; never use the `ennui_text` `Readout` for app UI. Code opens and closes screens, fills the resources that screens bind to, and answers their asks.
- The editor authors data only: no play-in-editor behavior and no scripting language. It writes scene files; apps load them.
- Aim the editor view at what you change, so the user sees it live.
- Each app commits every asset it uses in its own `assets/`. Never use Git LFS; a file of 50 MB or more goes into `.gitignore`, and you tell the user. Name assets as `assets/...` paths and load them through the library (`library_file`, `library_bytes`); no `include_bytes!` and no source-folder paths. The built-in fonts are the one exception.
- Values moved into shared code were chosen for one app: check each against every user before committing.

## Working

- Do the whole request as one job, without phases or stops between parts, and without asking design questions mid-task: decide from the known goals and continue. Stop only when nothing is left.
- Diagnose from the code and data; never ask the user for repro steps. Reason before running: trace the code, then run once to confirm; windows distract the user. Read the source before describing an app or crate, and read the code before relaying any agent finding.
- Verify in proportion: clippy on the touched crates and a targeted picture for a visible change; no full sweeps and no regression suite. A passing test proves nothing until it has failed against the old behavior; a flaky value needs 15 or more runs with the rows dumped.
- Never let a script take the computer down. Run check scripts and long jobs in the foreground under `timeout`, never in the background unwatched; a check that runs past a minute is stopped, not waited for. Stop only your own processes by id.
- Measure frame time with `just trace` span sums first, `--frames` for GPU passes; captures use `--step`, a fresh APPDATA, and a baseline in a worktree with its own target folder.
- Work on `main`; no branches unless asked. When several agents work at once, an agent that changes Rust works in its own git worktree with its own target folder, and rebases and pushes small finished commits to `main`, so its unfinished work never breaks the shared build. Stage only your own files. Commit and push finished work once it compiles and clippy is clean; never commit an agent's work in progress.
- Subagents use the `opus` model. Briefs give file and function anchors, name the checks instead of time budgets, allow git, and skip captures; the lead does the wiring and the on-screen check. Run parallel agents only for separate work.
- No tests of any kind: do not add them. No documentation beyond the README, the licenses and the skills unless asked.
