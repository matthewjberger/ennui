---
name: editor
description: The user starts it; it is the user's one entry point for interface work, and Claude also loads it before it drives the editor. The guide to building an app's screens and text tables and the command reference for driving the ennui editor through its localhost socket with editor-edit (create, set, ui, text, picture, notes and the rest). Use whenever the user says what they want on a screen (a menu, a HUD, a settings sheet, a dialog), asks how to work on an app's interface, or reports that a screen looks or acts wrong.
---

# Editor

The editor is the ennui scene editor. The user sees it; you drive it. Every change you make appears in the window at once, and every change the user makes reaches you as an event. It writes scene files; the app loads them.

## Start

1. In an app made with `just new`, `just edit [screen]` builds the editor from the ennui clone and opens it on the app's `project/` at that screen (`ui/home` by default). In the ennui repository, `just run editor` opens it on the template's project.
2. The editor writes its port to `<project>/.ennui/editor/port`. Talk to it with:

   ```
   <ennui>/target/play/editor-edit.exe --root <project> "line one" "line two"
   ```

   Each argument is one command line. With no arguments it reads lines from standard input. Run it from the Bash tool so that quotes inside a line stay intact. The line parser drops backslashes, so write every path with forward slashes.
3. Send `help` first in a new session, then `describe` to see every component and resource with its fields and defaults. Each line ends with the one-line description from `#[reflect(about = "...")]` on the type, so give every reflected app component and resource one. `describe` also lists the ones the app defines, marked "made by the app": the app writes them to `.ennui/components.txt` each time it starts, and `just edit` starts it with `--describe` before the editor opens. After you add a reflected resource to the app while the editor is open, rebuild and run `just run --describe`: it writes the file and closes, and the editor reads it by itself.

## How a request works

- All lines of one request run in order and undo as one step, so group the lines of one idea. Each scene keeps its own history.
- Each request runs alone from start to end; several agents can drive one editor, and their requests run one after the other. Start every request that needs a scene with `open <scene>`, and never count on the scene or choice that your last request left. A request of only `events` is answered at once.
- An error stops the request and names the line; the earlier lines stay applied. Read the message, fix the line and send again.
- `set` checks the component, the field and the value, and refuses what the component does not take.
- The reply ends with `events from the user:` when the user did something since your last request, for example chose an element, dragged one, made a change or wrote a note. Act on these.

## The scene model

- A scene is `<project>/scenes/<name>.scene`: screens under `scenes/ui/`, text tables under `scenes/text/`. It is text: entity rows with ids and names, `parent` and `use` lines, component leaves such as `Panel.pad 12`, and resource settings.
- Your edits go to the scene layer. The user's edits go to the user layer, `<name>.user.scene`, which is stronger. When a leaf the user set hides yours, leave it, or ask the user.
- `prefab <id> <name>` writes an element and all below it to `scenes/prefabs/<name>.scene` and makes it use that file, so later edits reach every user; `open prefabs/<name>` edits it.
- `project/settings.scene` holds the app's settings, such as `resource Interface theme "ui/theme"`; `setting` lists it, `setting <Name> <field> <value>` sets a line and `unsetting <Name> <field>` removes one.

## Screens and text

- A screen is a tree of entities. The root holds `Host` (`mount Screen` fills the window, `Inside` fills another element's rect; `reference` is the design size, 1280 by 720 by default, and every size below is in those pixels; `theme` names the entity whose `Theme` the tree uses; `layer` and `modal`). Each element holds `Panel` (size as `Fixed`, `Fill` or `Hug`, flow, pad, gap, clips, scrolls), an optional `Pin` (a spot on the parent as a share, nudged in pixels, with a pivot), `Style` (fill, edge, round, shadow, picture, and the hover, press, focus, off and lit moods) and `Text`. A `Theme` entity holds the colors, sizes and font the tree draws with. The controls (`Button`, `Toggle`, `Slider`, `Entry`, a dropdown, `Tabs`, `Repeat` for lists) are components on elements; `describe` lists them.
- Words come from text tables, scenes under `scenes/text/<table>.scene` with one `Line.words` entity per key, and reach an element through `Bind`: an entry binds a field such as `Text.words` to a source template, `{text.hud.lap}` for a table entry, `{Race.lap}` for a resource field, `{item.name}` for a list item, with a format (Plain, Whole, Decimals, Percent) and `back` for two-way writes. `Samples` on any entity above gives sample values, which the editor shows, since it never reads live data; the app reads the real sources.
- Motion is a `Sequence` entity (length, looping, cues) with `Channel` children (a target such as `box/Style.opacity` and keys with a time, a value and an ease), played by `Play` on the entity that owns it.
- The app opens a screen by name with `open_screen` from ennui-screens, fills the resources its bindings name, and answers its asks (`Button.ask "resume"` arrives in `Asks`). A bound field needs no code of its own.

## Commands for interfaces

- `open ui/<name>` edits a screen; the View pane is its Screen host. `create <id> "Name" ui host|panel|text|<control> under <parent>` makes elements with sensible defaults: `host` a Screen root that fills the view, `panel` a filled 240 by 160 box, `text` a label with the name, and `button`, `toggle`, `slider`, `entry` or `tabs` that control with a label. An element with `Text` needs a `Style` too; `create ... ui text` adds one.
- `ui aspect <w:h>|free` draws a frame of that aspect inside the view and lays the screen out inside it (the aspect chips on the top bar do the same). `ui show <screen>...` and `ui hide` draw other screens read-only over the open one, so a dialog can be checked over its menu. `ui sample "<source>" <value>` puts a sample value on the chosen element (or the scene's `Host`).
- The user clicks an element to choose it; a white rim marks the hovered element and an orange rim the chosen one. Dragging an element changes `Pin.nudge`; an element without a `Pin` is dropped before or after a sibling, which writes `Order`; the handles on the right edge, the bottom edge and the corner change `Panel.wide` and `Panel.tall` to `Fixed` pixels. Each drag is one undo step on the user layer. The scene tree reorders siblings the same way; `set <id> Order <n>` does it over the socket.
- `open ui/theme`, or any scene whose root holds a `Theme`, shows a sheet of every control in its states, drawn with that theme so edits show live.
- The Text pane lists every text in the project: each table entry as `text.<table>.<key>` with the bindings that use it, and every inline `Text.words` and literal `Bind` source. `text list [filter]` prints it, `text set <table>.<key> "<words>"` writes an entry, and `text move <id> <table>.<key>` moves an element's inline words into a table and binds them.
- The Timeline pane shows the chosen element's clip, with keys to add, drag and ease; `play <id> <clip>` plays a clip in the window and `play stop` stops them.
- Problems lists the binding problems (unknown components and fields, entries that no binding uses), the text table problems and the screen load problems.

## Other commands

`list`, `find`, `show <id>`, `set`, `unset`, `add`, `drop`, `delete`, `rename`, `name`, `parent`, `use`, `copy`, `paste`, `hide`, `unhide`, `lock`, `unlock`, `isolate`, `resource`, `unresource`, `undo`, `redo`, `history`, `save`, `revert`, `scene`, `open`, `watch`, `prefabs` (with `delete`, `rename`, `copy`), `layer`, `layout` and `quit`. `help` gives the syntax of each.

## Look at your work

`picture [name]` captures the window and answers with a file path; an absolute path writes the file there. Read that file to see the result. Take a picture after each group of visible changes, and check it before you tell the user you are done.

## Talk with the user

- `select <id>...` marks what you work on with a blue rim, and `selection` tells you what the user chose.
- `choose <id>...|none` sets the user's choice, as a click does.
- `notes` lists the user's open notes, `note "text" on <id>` adds one, and `done <number>` closes one.
- `say "text"` shows a line in the editor.

## Events on every prompt

`editor-edit --hook` prints what the user did since the last request in each open editor, and nothing when no editor is open or nothing happened. Each open editor writes a file named by its port under the user's ennui data folder (`%APPDATA%\ennui\editor\open` on Windows). The `UserPromptSubmit` hook in `.claude/settings.json` runs it, so you see the user's editor actions with each message.

## Finish

- `save` writes the changed layers. Save when a piece of work is complete, and tell the user what you saved.
- Use the socket for scene edits. A scene file written by another tool while the editor is open reloads in the editor, but edits through the socket keep undo.
