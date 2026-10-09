---
name: audit
description: The user starts it; Claude also reads its rules before writing code. Audit ennui code (the crates under crates/, the gallery and the editor under apps/, the template, and each app made from it in its own repository) against the project's data-oriented design rules, then fix what the audit finds in committed phases. Use when the user asks to evaluate, audit, review the architecture of, or clean up ennui, a crate or an app, or asks whether code follows data-oriented design, is well written, well architected, easy to extend, or grouped into the right crates.
---

# Audit

This skill repeats one loop: evaluate, report, wait for the go, fix in phases, verify, commit. Run the evaluation first and stop after the report unless the user already said to implement.

## Arguments

The arguments name the scope. It is one or more paths, or one of these words for a whole layer:

| Word | Paths | What it holds |
| --- | --- | --- |
| `crates` | `ennui/crates` | The `ennui-*` crates, grouped under `crates/engine/`, `crates/rendering/` (the renderer, the overlay pass and the window) and `crates/ui/`. |
| `apps` | `ennui/apps` | The gallery and the editor, each with its own crates under `apps/<app>/crates/` named `<app>-*`. |
| `template` | `ennui/template` | The app that `just new` copies. |
| `<app>` | `<app>` | An app made from the template, in its own repository, with its own crates under `crates/`. |

A path can be narrower, for example `ennui/crates/ui` or `ennui/apps/editor/crates/editor-core`. With no argument, audit `crates`. The words `implement`, `fix` or `go` in the arguments mean the user wants the fixes too.

The tools are the `audit` subcommands of the `ennui` command line tool. Run it with `cargo run -q --release --manifest-path <ennui>/tools/ennui/Cargo.toml -- audit <subcommand>`, where `<ennui>` is `.` from the ennui repository and the path to the ennui clone from an app repository; below, `ennui audit <subcommand>` stands for that line. Each subcommand reads the repository it runs in, so run it from the root of the repository that holds the scope.

## The layers of the repository

The same rules apply everywhere, but each layer also has its own questions:

- **Crates (`crates/`)**
  - Each crate is one feature with a `plugin` that gives `resources` and `systems`.
  - A crate under `crates/` never depends on an app.
  - Every library crate is linked into `crates/engine/ennui-dylib`: a `<crate>.workspace = true` line in its `Cargo.toml` and an `extern crate <crate_name>;` line (dashes become underscores) in its `src/lib.rs`. Apps with the `dynamic` feature link the crates through this one shared library, so that an app rebuild does not relink them. A crate that is missing is linked statically into each app instead, which makes builds slower and can give the app a second copy of the crate. When you add, rename, split or remove a crate, update both files in the same change. The `dylib` check finds the gaps. Derive crates (`proc-macro = true`) cannot be linked into a dylib and are skipped.
  - A crate with only one app user lives under that app with its prefix, and moves into `crates/` when a second user needs it. An app never depends on a crate of a different app.
  - Check the grouping: does a feature sit in the right crate, does `ennui` stay the small core, and does each GPU crate own one pass?
- **Apps (`apps/`, `template/` and app repositories)**
  - Each app is one crate with the same layer files inside `src/`, and its own crates under `crates/` for code that is big enough to stand alone.
  - Code that more than one app needs belongs in a crate under `crates/`, not copied between apps. The `duplicates` check across `apps` finds copies.
  - Screens are scene files under `project/scenes/ui/`, not widget trees built in code; code fills the resources that screens bind to and answers their asks.

## The rules

Hold every crate in scope to these rules:

1. All per-entity data goes into components.
2. All data that is shared across entities goes into resources.
3. All procedural logic and behavior goes into systems.
4. The code uses as few lines as is possible and reasonable to express each technique.
5. No function has a body of one line. A one-line function is never necessary: write its body where it is called and delete it. This covers wrappers that only forward to another function, getters, predicates, format helpers and systems that only call a command. When a system would only call a command, move the command body into the system and delete the command. A function pointer that a registry keeps, such as a reflect read or write, becomes a closure with no captures at the place that fills the registry. Two exceptions stay: a `pub fn` that other crates call as API vocabulary (for example `on`, `before`, `insert_resource`), and a one-line expression, usually a lookup, that would otherwise be written out at 4 or more call sites (for example `name_at` in ennui-document). A function whose body only forwards to another function is never an exception. The scanner applies both exceptions.
6. Crates group related code. A crate has one clear job, its dependencies point one way, and no code sits in a crate only because it was convenient.
7. The rules in CLAUDE.md apply: no comments, no abbreviations, no `unsafe`, no `mod.rs`, no `#[allow(...)]`, `nalgebra_glm` only, `pub` only when another crate uses the item, every crate manifest has a `[lints]` section.

## The layers of an ennui crate

Every crate uses the same files. Put each thing in its layer:

| Layer | Holds |
| --- | --- |
| `components.rs` | Per-entity data types. No logic. |
| `resources.rs` | Shared state, one value for the world. UI handle structs are fine here. A map keyed by `Entity` is not. |
| `data/` or `data.rs` | Plain types, tables and constants that are not components or resources: enums, rows of a table, job structs, the `impl` of a trait for a data type. |
| `systems/` | Only functions that the scheduler calls. Their parameters are `Res<...>`, `ResMut<...>`, `View`, `Peek`, `Mut`, `Glance` and `Later`. A tuple of 2 to 5 `ResMut` is one parameter; share it as a type alias. Never `Rows`: a system that holds `&mut Storage` runs alone and cannot join a parallel batch. No helper functions, structs, constants or impls. A system is at most 60 lines. |
| `commands.rs` or `commands/` | Helpers that change storage or resources, called from systems. They take the columns they write or a `Later` queue for spawns and structural changes, never `&mut Storage`. |
| `queries.rs` or `queries/` | Helpers that only read and compute. |
| `theme.rs` | Tuning numbers, colors, sizes and timings. Systems, commands and queries name these constants and do not hold magic numbers. |
| `plugin.rs` | Resource setup and the system schedule. |

A recipe table written as code is data. Keep its numbers together in one place, but they do not need one constant each.

## Step 1: evaluate

Do not change code in this step.

1. Run the scanner from the root of the repository that holds the scope:

   ```
   ennui audit scan <paths...>
   ```

   Give it the paths in that repository, for example `crates` or `apps` in `ennui`, or `. crates` in an app repository. The scanner reads only the repository it runs in. It reports these checks:

   | Check | Finds |
   | --- | --- |
   | `rules` | Comments, `#[allow(...)]`, `unsafe`, `glam`, `mod.rs` files and manifests with no `[lints]`. |
   | `layers` | Types, impls or helper functions in system files. A type alias for a system parameter, in any file of the crate, counts as a system parameter. |
   | `long` | Systems over 60 lines. |
   | `placement` | Resources keyed by entity. Read the code before reporting one: state belongs in a component, but a private index derived from a component and rebuilt from its changes (for example `ennui-scene` `Hierarchy`, the children lookup built from `ChildOf`) is correct and stays. |
   | `logic` | Functions and inherent methods in `resources` and `data` files that branch, loop or compute. |
   | `literals` | Tuning numbers in systems, commands and queries, as per-file counts. Add `--detail` for the lines. |
   | `chains` | Three or more `set` calls on one entity. |
   | `self_methods` | Methods that take `&mut self`, functions with a `&Self` or `&mut Self` parameter, and functions that take `self` by value without building `Self`. Functions that an outside trait requires (`Drop`, `DerefMut`, `Hasher`, winit handlers, `Default`, `From`) are skipped by name. |
   | `traits` | Traits defined in the repository whose functions take `self` or `Self`: behaviour chosen by type. |
   | `one_liners` | Functions with a body of one line. Each one gets inlined at its callers and deleted. `main.rs`, `lib.rs` and `plugin.rs` are skipped. |
   | `storage` | Any `Rows`, and public helpers that take `&mut Storage`. A `Rows` system runs alone and never joins a parallel batch. `ennui-ecs` and `ennui` are skipped. |
   | `hash_walks` | Functions that walk a HashMap or HashSet and also spawn, attach, set, despawn, send or ask a worker, with no sort. It matches names only, so a Vec field with the same name as a hash field elsewhere in the crate is a false positive, and a walk in one function that another function writes is missed. |
   | `dependencies` | Manifest entries that the source never names. An optional dependency that a feature turns on with `dep:` counts as named. |
   | `edges` | Crates under `crates/` that depend on an app, and crates that depend on a crate of a different app. |
   | `dylib` | Library crates that `crates/engine/ennui-dylib` does not depend on or does not name with `extern crate`. It runs only in the ennui repository. |
   | `crate_layers` | Crates outside the UI layer that depend on a UI crate (a crate under `crates/ui/`). UI crates may build on each other. `ennui-dylib` and `ennui-sets` are permitted. It runs only in the ennui repository. |
   | `visibility` | `pub` items in library crates that no other crate of the repository uses. App binaries are skipped, because there `pub` and `pub(crate)` are the same. In `crates/`, such an item may be API that an app in another repository uses: the scanner cannot see it, so leave it `pub` unless you know it has no user. |
   | `duplicates` | The same six-line window in two places. Windows found only in `main.rs` files are marked `(plugin setup)`. |

   Use `--only a,b` to pick checks, and `--limit N` or `--window N` to tune them.
2. The scanner finds candidates. It does not decide. Read the source of every crate in scope before you say anything about it. Never describe a crate from its name, its flags or the scanner output alone.
3. For a large scope, send parallel read-only agents, one per group of related crates. Tell each agent the rules, the layer table and what to return: bugs with file and line, rule violations, duplicate code, dead code, systems to split, and crate grouping problems.
4. Look for what the scanner cannot see:
   - Bugs: wrong state transitions, missing resets on a reload, off-by-one errors, systems that run on the wrong screen.
   - Per-entity state held in resources, and shared state copied onto entities.
   - Logic inside components, resources or data impls that the `logic` check cannot see, such as a closure table or a macro.
   - System order that only works because of the plugin line order.
   - Features that no app uses. List them for the gallery, not for removal.
   - The same technique written twice in different words.
   - Dead code: fields never read, parameters always given the same value, placeholders that nothing fills, unused dependencies.
   - Crate grouping: a crate that holds code for another concern, a common crate that grows into a dumping ground, dependency edges that point the wrong way.
5. Report in this order:
   - A verdict on each question: well written, well architected, easy to extend, consistent with data-oriented design, and are the crates grouped well. Give the reasons.
   - Numbered bugs, each with the file, the line and the effect.
   - Rule violations grouped by rule.
   - Duplicates and dead code.
   - Crate regrouping proposals: what moves where and which dependency edges change.
   - The phase plan from Step 2 with the size of each phase.
6. Stop and wait for the user to say to implement, unless the arguments already said so.

## Step 2: fix in phases

Work on the branch the user names, usually `main`. Finish and commit one phase before the next:

1. Bugs.
2. Placement: per-entity data into components, shared data into resources. Attach components at startup and read them back with one query per system.
3. Layers: helpers out of system files into commands and queries, types into data, tuning numbers into the theme.
4. Duplicates, dead code, long systems, one-line functions, `pub` that is not needed, `set` chains into `attach`.
5. Crate regrouping.

Keep behavior the same unless the change is a bug fix. When a fix replaces a function, delete the old one in the same change, and delete helpers, imports and constants that only it used.

### Running a phase with parallel agents

A large phase goes faster with one agent for each group of crates. Give each agent its own files, and name the files it must not touch:

- The files that agents collide on are each app's `main.rs` and `Cargo.toml`, the workspace `Cargo.toml`, and a crate's `lib.rs` prelude. Give each of these to one agent, or run the agents that need them one after the other.
- Tell each agent to build only its own packages with `-p`, and to wait and retry when a build fails in a file it does not own. Another agent's work in progress causes that failure.
- Check the free disk space before a large parallel build. Many release and debug targets together can fill the drive.
- Review each agent's report before you commit: read the behavior changes it lists, and grep its diff for comments, `#[allow`, `unsafe` and abbreviations.

### Tools for the fixes

- `ennui audit move <crate folder> <source file> <target file> <names...>` moves functions, structs, enums, consts and types between files of one crate. The file paths are relative to the crate's `src`. It makes the moved items `pub(crate)`, copies the source imports and imports the items back into the source.
- `ennui audit tidy <files...>` flattens `use` statements and removes duplicates after a move.
- `cargo fix --lib -p <crate> --allow-dirty --allow-staged` removes the unused imports that moves leave behind. Run it, then `cargo fmt`, then clippy.
- For many mechanical edits in one change, write a Python script with exact string or regex replacements that assert each match count. Write the script with the Write tool, because a heredoc in the Bash tool can break on quotes.
- After you narrow `pub` to `pub(crate)`, clippy reports types that a public function still exposes. Make those types `pub` again.

### Verify each phase

1. `cargo fmt` for the crates you touched.
2. `cargo clippy --all-targets --message-format short` for every crate in scope, with no warnings.
3. Compare frames with the last commit before the phase. Keep the baseline in a worktree with its own target folder, never the main target. Run both commands from the root of the repository:

   ```
   ennui audit baseline --commit <commit before the phase>
   ennui audit shots --label <phase> --apps gallery,editor
   ```

   An app repository builds the crates through its path to the ennui clone. To compare a change to the crates in an app, make a baseline of the ennui clone beside a baseline of the app, so that the baseline app builds the baseline crates, and give the commands full paths.

   `shots` gives each app only the arguments named for it with `--pass <app>=<argument>`, once for each argument; the template needs `--pass template=--play` to skip its title screen.

   `shots` builds both trees, runs each app with `--step 0.0166` and a fresh data folder (APPDATA on Windows, XDG_DATA_HOME on Linux), and prints the pixels that differ by more than 32 and by more than 0, and the box around the changes. `ennui audit compare <first.png> <second.png>` prints the same line for any two pictures. Measure the noise of an app first: run the same build twice. With `--step`, `crew_take` waits for the jobs in flight and returns them in the order they were asked, so worker results arrive in the same frame on every run. With `--step` the window ignores keyboard and mouse, so moving the mouse during a capture does not change it. To test a key or a click, script it: `--press KeyJ@60` presses J on frame 60, `--press KeyW@60-120` holds W, `--click 640,360@60` clicks there, `--point 640,360@60` only moves the pointer, `--wheel 1@30-60` turns the wheel one step on each of those frames, `--drag Left:640,360>1040,360@80-110` holds a button and moves the pointer in a line over those frames, and `--type "drop 3@30"` types the text before the last `@` into whatever reads typed text on frame 30, such as a focused text field or the console. Each flag can be given more than once. A difference over 32 must have a cause that you can name. At the end, remove each baseline with `git -C <repository> worktree remove --force <path>`.
4. Start every program of the repository once. The order check, the shader composer and the argument parser run at start, so a start catches what clippy cannot:

   ```
   ennui audit starts --frames 30
   ```

5. For an app, capture a frame and look at it:

   ```
   ennui audit capture --app <package> --name <label> --frame 120
   ```

   To start on a different screen, pass `--main <path to main.rs> --from <text> --to <text>`. The command puts the file back after the run. Read the picture it prints and check that the interface renders. The app also writes a census next to the picture. Read its first section, the queries that matched nothing: a system that should act on entities but matched none has a missing component or an order problem. Register the values of new components and resources with `show::<T>` so the census shows them.
6. When you add a new check, such as a compile-time assert or a macro, make it fail once on purpose, then restore it.
7. To prove an order rule, move the earlier plugin line to the end of the app and capture: the frame must not change. Then remove the rule and capture again: the frame must change. A still screen hides order problems, so pick an app where things move.

### Commit

Commit after each phase. Write the message to a temporary file and run `git commit -F <file>`. Use a `type(scope): summary` subject and a short body in plain words, with no attribution lines and no em-dashes. Push only when the user asked for it.

## Judgement notes

- UI handle structs in resources, such as a panel with `Entity` fields, are shared state for one screen and can stay.
- A GPU guest pass lives in `systems/pass.rs` with its pipeline, uniforms, builder and render callbacks, as in `ennui-wgpu-overlay`. A pass with several stages keeps them in `systems/pass/`, one file per stage with its shaders beside it. The scanner skips `pass.rs` and `systems/pass/`. Keep ordinary systems out of them.
- Small epsilon guards such as `1.0e-4` are not tuning numbers.
- A `&mut self` method that an outside trait requires, such as `DerefMut`, `Drop` or a winit handler, can stay. An inherent `&mut self` method becomes a free function over the data.
- A parameter written as `held: &mut Self` is a method in disguise, and so is a function that takes `self` by value but does not build a `Self`, such as a job's `work(self, ...)`. Both are object oriented.
- Do not define traits whose functions take `self` or `Self` to choose behaviour by type, such as a job trait with `order`, `fill` and `work`. Store the work as data: plain rows grouped by kind (one queue or pool per kind, or an enum of a few variants with one `match`), processed by one known function for that kind, ideally a batch over the rows. A function passed once per pool or per call is a plain parameter and is fine. Never store a function pointer inside each item: that is a hand-written vtable, the same thing as OOP dispatch. Marker traits without functions can stay. The ECS core's type machinery in `ennui-ecs` (`Param`, `IntoSystem`, `Bundle`, `Take`, `Join`), which turns plain functions into systems and tuples into queries, and the derived reflection in `ennui-reflect` (`Reflect`, which turns a component's fields into text for scenes, the inspector and `--describe`) are generated type plumbing, not logic, and are skipped by the `self_methods` and `traits` checks. The line is: logic is never chosen by type; generated type plumbing may be.
- Builders that take `self` by value and return `Self`, such as `Frame` and `Fitting`, are the builder style and can stay. So can constructors that only fill fields and return `Self`: they are syntax sugar for free functions. Do not convert them.
- The `dependencies` check misses names used only through macros or features. Check each entry before you remove it.
- rustfmt splits long calls over many lines. To shorten a system, move a coherent step into a command or a query, not single lines.
- When a system and a command need the same `&mut` value, reborrow it with `&mut *value`.
- Order systems with `before(earlier, later)` rules next to the systems, and use `grouped(Group, step)` when one rule covers many systems. Code that only works because of the plugin line order in `main.rs` is a bug. Moving a plugin line must never change behavior.
- A rule between two crates names a group that the earlier crate owns, never a system function of another crate. A system stays private to its crate. Add rules only for orders that change behavior. An event reader always sees the previous frame's events, so a sender and a reader never need a rule.
- When a component only works together with others, declare it with `require::<T, Needed>` in the plugin's resources, and do not list the defaults at each spawn site. A missing component makes a query skip the entity with no error. Add the rule only when the default means the same as the value every spawn site gives.
- A usable feature that no app uses gets a page in the gallery that exercises it, not removal. Read the feature and check that it works first; a demo often finds a bug. Remove only code that is dead: nothing calls it and it has no use a demo could show.
- Make runs repeatable. A random source that is not seeded, or a list shown in HashMap order, makes every capture differ and hides real changes. Seed the source and sort the list. A HashMap or HashSet walk that calls `spawn`, `attach` or `set` puts entities into archetype rows in a random order, and systems iterate rows in order, so a seed or job index taken from row order changes too. Walk a BTreeMap, or collect and sort, before a storage write. When two censuses list the same entities in a different order, look for such a walk.
- Prefer a generic system over many copies. Examples: the `on_event::<E, _>` gate in `ennui::events` and `reset::<R>` and `clear::<C>` in `ennui::systems`, and `closing_footer` with the `Closer` component for close buttons.
