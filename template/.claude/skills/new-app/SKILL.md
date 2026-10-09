---
name: new-app
description: Start app work from where the session runs. In the ennui repository, make a new ennui app from the template as its own repository beside the ennui clone, and run it. In an app repository that just new made, work on that app with the editor. Use when the user starts it, asks for a new app, wants to try the template, or asks how an app is set up against ennui.
---

# New app

## First, find where the session runs

Look at the repository of the working folder (`git rev-parse --show-toplevel`) before you do anything else. The folder decides the task; do not ask the user which task they want.

- **The ennui repository** (its root has `crates/engine/ennui-dylib` and `template/`, also when the session starts in `template/`): the user wants a new app. Make it from the template as "Make a new app" tells. Take the app name from the user's message; if the message has no name, ask for the name only, then do all the steps without more stops. When the app is made, open it with `just edit` in the new folder.
- **An app repository** (its justfile sets `app :=` and an `engine :=` path to the ennui clone, as `just new` writes it): the user wants to work on this app. Do not make a new app. Load the `ennui:editor` skill and follow it for this app: read the app's `CLAUDE.md`, its screens and its code, run `just edit` to open the editor on it, and then do what the user asks in the editor. If the user asked for nothing more, open the editor and tell the user that it is ready.
- **Another folder**: tell the user that the skill runs in the ennui repository or in an app repository, and stop.

The ennui repository holds `template/`, a complete app: an editor project with a home screen and a settings screen with a two-way bound slider, a text table, a theme, and a back key on Escape. It is its own Cargo workspace that names ennui as `..`, so it runs straight from a clone.

## Run the template

In a clone of `ennui`:

```
cd template
just run
```

The first build compiles the crates into `template/target` (some minutes); later builds link the shared library and are fast. `just edit` builds the editor from the same clone and opens it on the template's project.

## Make a new app

A new app is a copy of `template/` in its own folder, with its names changed and its ennui path pointed at the ennui clone. Nothing else changes.

1. Choose the folder: beside the ennui clone, or where the user says. The app name is lowercase letters, digits and hyphens, starting with a letter.
2. Run `just new <folder>` in `ennui` (a relative folder starts at the folder where `just` runs). It:
   - copies `template/` without `target/`, `.ennui/` and user layers;
   - replaces the names `template`, `Template` and `TEMPLATE` in code, manifests, scenes and the justfile;
   - changes the ennui path: `engine := ".."` in the justfile, every `path = "../crates/...` in `Cargo.toml` and the `@../` imports in `CLAUDE.md` become the relative path from the new folder to the ennui clone;
   - runs `git init`, `just sync` and a first commit.
3. In the new folder, `just run` runs it and `just edit` opens the editor on it. The `editor` skill covers building out its screens.

If `just new` cannot be used, do the same steps by hand: copy the folder, rename as above, and change only those kinds of path. Do not copy ennui crates into the app; the app builds them from the clone.

Create a GitHub repository for the app only when the user asks.

## Keep the template in step

The template is not a member of the ennui workspace, so work on the crates does not build it. When work changes the workspace dependencies (a new or renamed crate, a version), profiles, lints, the toolchain or the lock, run `just sync` in `template/` and commit the result with the change, so that a new app starts from current settings. When work changes an API the template uses, fix the template in the same commit; `just check` in `template/` builds it.
