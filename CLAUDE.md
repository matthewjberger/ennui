@RULES.md
@INTERFACE.md

# Working on ennui

The workspace holds `crates/` (grouped as `crates/engine/`, `crates/rendering/` and `crates/ui/`) and the apps under `apps/`: `apps/gallery`, a tour of every widget, and `apps/editor`, the scene editor with its own crates under `apps/editor/crates`. `examples/` holds example apps (`examples/showcase`), each with its own `project/` and `assets/`.

- `just lint` runs the format check and clippy on the workspace and the tool; `just fmt` formats them.
- `just run <app>` runs an app of `apps/` (`just run gallery`, `just run editor`) (the editor opens on the template's project), and `just pick` lists them.
- Every library crate under `crates/` is linked into `crates/engine/ennui-dylib` (a `<crate>.workspace = true` line and an `extern crate` line). When you add, rename or remove a crate, update the dylib and `[workspace.dependencies]` in the same change, then run `just sync` in `template/`.
- `tools/ennui` is the `ennui` command line tool that every justfile recipe calls: run, trace, edit, export, check, lint, fmt, sync, new, pick and audit. It is its own workspace with its own `target/`. A change to a recipe's behavior is a change to the tool, not to a justfile.
- `template/` is a complete app in its own workspace that names this repository as `..`. It is not built by `just lint`: run `just check` in `template/` when work touches what it uses, and fix it in the same commit.
- The skills are in `.claude/skills/` and the editor hook is in `.claude/settings.json`; Claude Code loads both from the repository, with nothing to install. `just sync` copies the skills into an app repository and points its hook at this clone.
