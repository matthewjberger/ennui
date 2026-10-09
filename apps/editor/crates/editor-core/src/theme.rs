pub(crate) const LABEL_LENGTH: usize = 60;
pub(crate) const LISTS: u32 = 6;
pub(crate) const SPACING_SHARE: f32 = 0.75;
pub const PICTURE_DELAY: u64 = 3;
pub(crate) const REVEAL_SECONDS: f32 = 2.0;

pub(crate) const HELP: &str = "\
ennui editor commands. Lines in one request run in order and undo as one step.
Values: numbers, words, \"text\", @id for an entity, [list], {field value}, Variant{...}.
  help                                   this text
  describe [Name]                        components and resources with their fields
  list [filter]                          the entity tree with components
  find <word>                            entities whose id, name or components match
  show <id>...                           authored leaves and live values of entities
  create <id|auto> [\"Name\"] [under <parent>]
         [with <Component>]... [tag \"word\"] [use \"scenes/prefabs/x.scene\"]
         [ui host|panel|text|<control>]  a ui element with sensible defaults, under a parent in a ui scene:
                                         host is a Screen root, panel a filled box, text a label with the name,
                                         and button, toggle, slider, entry or tabs that control plus a label
  set <id> <Component.field> <value>     author a leaf, for example set card Panel.pad 12
  unset <id> <Component.field>           remove a leaf so the default comes back
  add <id> <Component>                   give an entity a component with default values
  drop <id> <Component>                  take a component away
  delete <id>...                         delete entities and everything below them
  rename <id> <new id>                   change an id and every reference to it
  name <id> \"Name\"|none                  change the shown name
  parent <id> <parent>|none              move an entity in the tree
  use <id> \"scenes/other.scene\"|none     place another scene below an entity
  prefab <id> <name>                     save an entity and all below it as scenes/prefabs/<name>.scene and use it there
  copy <id> [new id]                     copy an entity and everything below it
  resource <Name> [field value]          show or author a resource setting
  unresource <Name> <field>              remove a resource setting
  setting [<Name> <field> value]         show or author an app setting in settings.scene
  unsetting <Name> <field>               remove an app setting
  undo [n] | redo [n] | history [n]
  save                                   write the changed layers to disk
  revert                                 take back the unsaved changes
  scene save|copy|rename <name>          save the scene under a new name, copy it, or rename it
  paste                                  paste the entities that the user copied, with fresh ids
  hide|lock <id>... | unhide|unlock <id>...|all | isolate <id>...
                                         editor-only: hidden and locked entities cannot be picked
  open <scene>                           open scenes/<scene>.scene, or start a new one that the first save writes
  prefabs                                every prefab in scenes/prefabs with how many entities of the open map use it
  prefabs delete|rename|copy <name> [<new>]
                                         the same for scenes/prefabs/<name>.scene; delete is refused while any
                                         scene uses it, rename points every use line at the new file
                                         file moves and deletes have no undo; reference edits in the open scene do
  text [scene|user|all]                  the scene text of a layer or of the composed scene
  text list [filter]                     every text in the project: text table entries (scenes/text/<table>.scene)
                                         with the bindings that use them, and the inline Text.words and literal
                                         words in Bind sources of every scene, with the scene and entity
  text set <table>.<key> \"<words>\"       write a table entry, in the open table scene as an edit, otherwise on disk
  text move <id> <table>.<key>           move an entity's inline text into a table and bind the field to
                                         {text.<table>.<key>}; a Text.words gets a Bind entry, a Bind source is replaced
  ui aspect <w:h>|free                   frame the View pane to an aspect (16:9, 21:9, 4:3, 9:16) while a ui scene is
                                         open; Screen hosts lay out inside the frame; free uses the whole pane
  ui show <screen>... | ui hide           show screens from scenes/ui read-only over the open map at their real size
  ui sample \"<source>\" <value>           put a sample value for a bound source on the chosen entity (or the scene's
                                         Host), for example ui sample \"Race.lap\" 3; bindings never read live data here
  play <id> <clip> | play stop           play a Sequence clip on an entity in the editor window, or stop every one
  layer scene|user                       which layer your edits go to (default scene)
  select <id>...|none                    mark entities as the ones you work on
  selection                              what the user has chosen
  choose <id>...|none                    choose entities for the user, so the inspector shows them
  notes | note \"text\" [on <id>] | done <number>
  say \"text\"                             show a message to the user
  picture [name]                         capture the window and give back the file path; an absolute
                                         path with or without .png writes the file there
  watch                                  the folders the editor watches, the files that changed on disk
                                         lately and what the editor reloaded for them
  wait [frames]                          let frames run before the next line
  events                                 only report what the user did
  layout reset                           put every pane back where it started
  layout show <pane>                     bring a pane to the front: Scene, History, View, Inspector, Problems,
                                         Messages, Text, Timeline, Screens
  quit                                   close the editor without saving";

pub(crate) const TOP: f32 = 40.0;
pub(crate) const FOOT: f32 = 26.0;
pub(crate) const LABEL: f32 = 118.0;
pub(crate) const NUMBER_STEP: f32 = 0.01;
pub(crate) const WHOLE_STEP: f32 = 0.25;
pub(crate) const TURN_STEP: f32 = 0.5;
pub(crate) const SCALE_STEP: f32 = 0.005;
pub(crate) const PLACE_SHARE: f32 = 0.01;
pub(crate) const RANGE_PIXELS: f32 = 200.0;
pub(crate) const NEST: f32 = 12.0;
pub(crate) const LIST_MOST: usize = 32;
pub(crate) const TINT_NEAR: f32 = 1.0e-4;
pub(crate) const MARKS: ennui_ui_controls::prelude::Marks = ennui_ui_controls::prelude::Marks {
    shut: ennui_ui_icons::prelude::icons::CHEVRON_RIGHT,
    open: ennui_ui_icons::prelude::icons::CHEVRON_DOWN,
};
pub(crate) const SCALE_NAMES: [&str; 2] = ["scale", "size"];
pub(crate) const PLACE_NAMES: [&str; 2] = ["position", "offset"];
pub(crate) const NO_FILES: &str = "No matching files under assets";
pub(crate) const MORE_TIP: &str = "Component menu: reset, copy or paste";
pub(crate) const REMOVE_TIP: &str = "Remove this component";
pub(crate) const WIDE_RANGE: f32 = 100_000.0;
pub(crate) const TEXT_ROOM: usize = 120;
pub(crate) const TOLD_LIFE: f32 = 6.0;
pub(crate) const HINT: &str = "Click to choose, Shift adds. Right click for actions. Ctrl P searches and runs anything, F1 lists every command";
pub(crate) const SHELL_ORDER: u32 = 3;
pub(crate) const LOCALHOST: &str = "127.0.0.1:0";
pub(crate) const PREFABS: &str = "scenes/prefabs";
pub(crate) const NEW_SCENE: &str = "New scene";
pub(crate) const PREFAB_STEM: &str = "prefabs";
pub(crate) const MAKE_PREFAB: &str = "Make prefab";
pub(crate) const OPEN_PREFAB: &str = "Open prefab";
pub(crate) const ADD_SETTING: &str = "Add a scene setting";
pub(crate) const SETTINGS_HINT: &str = "Nothing is chosen, so these are the scene settings.";
pub(crate) const DRAG_HINT: &str =
    "Drop on an entity to put it below that one, or on empty space to take it out";
pub(crate) const GRID_CELL: f32 = 1.0;
pub(crate) const ASSETS: &str = "assets";
pub(crate) const EXPAND_ALL_TIP: &str = "Expand all";
pub(crate) const COLLAPSE_ALL_TIP: &str = "Collapse all";
pub(crate) const APP_SETTINGS: &str = "settings.scene";
pub(crate) const DENSE: [f32; 10] = [19.0, 5.0, 4.0, 13.0, 15.0, 24.0, 15.0, 15.0, 28.0, 15.0];
pub(crate) const SCENES_TIP: &str = "Open another scene, save as, duplicate, rename or revert";
pub(crate) const CONTEXT_AWAY: f32 = -1.0e4;
pub(crate) const FILTER_ROOM: usize = 40;
pub(crate) const CLIPBOARD: &str = "clipboard.scene";
pub(crate) const LOG_MOST: usize = 500;
pub(crate) const HAPPENED_MOST: usize = 500;
pub(crate) const DIALOG_ORDER: u32 = 7;
pub(crate) const PALETTE_ORDER: u32 = 8;
pub(crate) const SEARCH_TIP: &str = "Search and run anything (Ctrl P)";
pub(crate) const ADDS_TO_CHOICE: &str = "Adds it to the choice";
pub(crate) const MARKS_DONE: &str = "Marks it done";
pub(crate) const IN_THIS_MAP: &str = "In this map";
pub(crate) const NOT_SET: &str = "Not set";
pub(crate) const DIALOG_WIDE: f32 = 380.0;
pub(crate) const DIALOG_MARGIN: f32 = 24.0;
pub(crate) const DIALOG_SCRIM: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.78);
pub(crate) const STORY_MOST: usize = 100;
pub(crate) const RESTORED_LIFE: f32 = 12.0;
pub(crate) const STEADY_STEP: f32 = 0.1;
pub(crate) const WINDOW_TITLE: &str = "ennui editor";
pub(crate) const WORKTREES: &str = ".claude/worktrees/";
pub(crate) const UNSAVED: &str = " *";
pub(crate) const RECENT_HINT: &str = "Recent";
pub(crate) const FOLDER_ACTIONS: [(&str, crate::data::SceneChoice); 4] = [
    (
        "Save as",
        crate::data::SceneChoice::Name(crate::data::Naming::SaveAs),
    ),
    (
        "Duplicate",
        crate::data::SceneChoice::Name(crate::data::Naming::CopyScene),
    ),
    (
        "Rename",
        crate::data::SceneChoice::Name(crate::data::Naming::RenameScene),
    ),
    ("Revert", crate::data::SceneChoice::Revert),
];
pub(crate) const ROW_KNOB: f32 = 18.0;
pub(crate) const ROW_ICON: f32 = 12.0;
pub(crate) const NAMING_TITLES: [(crate::data::Naming, &str, &str); 4] = [
    (
        crate::data::Naming::SaveAs,
        "Save the scene as",
        "scene save",
    ),
    (
        crate::data::Naming::SaveLayout,
        "Save the layout as",
        "layout save",
    ),
    (
        crate::data::Naming::CopyScene,
        "Duplicate the scene as",
        "scene copy",
    ),
    (
        crate::data::Naming::RenameScene,
        "Rename the scene to",
        "scene rename",
    ),
];
pub(crate) const COPY_SUFFIX: &str = "_copy";
pub(crate) const CONFIRM_CHOICES: [&str; 2] = ["Delete", "Cancel"];
pub(crate) const CONFIRM_ROOM: f32 = 340.0;
pub(crate) const NO_UNDO: &str = "Undo cannot bring them back.";
pub(crate) const LEAVE_QUESTION: &str = "Save the changes to this scene first?";
pub(crate) const LEAVE_CHOICES: [&str; 3] = ["Save", "Discard", "Cancel"];
pub const SUMMARY_CHOICES: [&str; 2] = ["Undo", "Close"];
pub(crate) const SUMMARY_LIFT: f32 = 80.0;
pub(crate) const ABOVE_PINS: u32 = u32::MAX;
pub(crate) const CARD_WIDE: f32 = 460.0;
pub(crate) const CARD_DOWN: f32 = 36.0;
pub(crate) const NOTE_ROOM: usize = 240;
pub(crate) const NOTE_CARD_TITLE: &str = "Note for Claude, Enter keeps it, Escape drops it";
pub(crate) const SHOW_CARD_TITLE: &str = "Show Claude this view, Enter sends it, Escape drops it";
pub(crate) const NOTE_ARMED: &str = "Click the element the note for Claude is about";
pub(crate) const ASKER_CHOICES: [&str; 2] = ["OK", "Cancel"];
pub(crate) const LINE_GAP: f32 = 2.0;
pub(crate) const PICK_FILE_TIP: &str = "Pick a file from the project or the bundled assets";
pub(crate) const EMPTY_OUTLINE: &str = "Empty. Press + or ask Claude.";
pub(crate) const ADD_COMPONENT: &str = "Add a component";
pub(crate) const LOG_EMPTY: [&str; 2] = ["No problems.", "Nothing yet."];
pub(crate) const LOG_EDGE: f32 = 56.0;
pub(crate) const UNSAVED_TITLE: &str = "Unsaved changes";
pub(crate) const SCENELESS: [&str; 13] = [
    "help",
    "setting",
    "unsetting",
    "describe",
    "open",
    "prefabs",
    "quit",
    "events",
    "say",
    "wait",
    "picture",
    "layout",
    "watch",
];
pub(crate) const EDITOR_WATCH: &str = "editor";
pub(crate) const PROJECT_WORK: &str = ".ennui";
pub(crate) const RELOADED_MOST: usize = 20;
pub(crate) const RELOADED_COMPONENTS: &str =
    "reloaded the app's components from .ennui/components.txt";
pub(crate) const NOTHING_CHANGED: &str = "no file changed on disk since the editor started";
pub(crate) const SCOPES: [(char, &str); 4] = [
    ('>', "Commands"),
    ('@', "Scene"),
    ('%', "Prefabs"),
    (':', "Settings"),
];
pub(crate) const FRESH_SCENE_TIP: &str = "Start a new scene, the first save writes it";
pub(crate) const REVERT_TIP: &str = "Take back the unsaved changes";
pub(crate) const PANES: [(&str, &str); 9] = [
    ("Scene", "The scene tree"),
    ("History", "History, click a step to undo or redo to it"),
    ("View", "The view"),
    ("Inspector", "The chosen entities, or the scene settings"),
    ("Problems", "Problems: errors and warnings"),
    (
        "Messages",
        "Messages: what Claude, the app and the editor said",
    ),
    (
        "Text",
        "Text: every text in the project, table entries and inline words, with where each is used",
    ),
    (
        "Timeline",
        "Timeline: the channels and keys of the chosen entity's clip on a time ruler",
    ),
    (
        "Screens",
        "UI screens of this project, click one to show it over the open map",
    ),
];
pub(crate) const GROUPS: [&[usize]; 5] = [&[0], &[8, 1], &[2], &[3], &[4, 5, 6, 7]];
pub const UI_FOLDER: &str = "ui";
pub const TEXT_FOLDER: &str = "text";
pub const THEME_SCENE: &str = "ui/theme";
pub const ASPECTS: [(&str, Option<[f32; 2]>); 5] = [
    ("16:9", Some([16.0, 9.0])),
    ("21:9", Some([21.0, 9.0])),
    ("4:3", Some([4.0, 3.0])),
    ("9:16", Some([9.0, 16.0])),
    ("free", None),
];
pub(crate) const HANDLE: f32 = 8.0;
pub(crate) const UI_ROUNDING: f32 = 1.0;
pub(crate) const REORDER_SHARE: f32 = 0.3;
pub(crate) const UI_RIM_ORDER: u32 = u32::MAX - 1;
pub(crate) const BIND: &str = "Bind";
pub(crate) const SAMPLES: &str = "Samples";
pub(crate) const TEXT: &str = "Text";
pub(crate) const LINE: &str = "Line";
pub(crate) const ORDER: &str = "Order";
pub(crate) const PANEL: &str = "Panel";
pub(crate) const PIN: &str = "Pin";
pub(crate) const STYLE: &str = "Style";
pub(crate) const HOST: &str = "Host";
pub(crate) const UI_PANEL_SIZE: [f32; 2] = [240.0, 160.0];
pub(crate) const UI_PANEL_PAD: f32 = 8.0;
pub(crate) const UNBOUND_ROOM: f32 = 300.0;
pub(crate) const UNBOUND_HINT: &str = "add a sample above or on this entity to show a value here";
pub(crate) const VIEW_PANE: usize = 2;
pub(crate) const LOG_PANES: [usize; 2] = [4, 5];
pub(crate) const LOG_GROUP: usize = 4;
pub(crate) const LOG_SHARE: f32 = 0.22;
pub(crate) const LAYOUTS: [(&str, &str); 3] = [
    (
        "Default",
        "(across 0.200 (down 0.500 (tabs 0 Scene) (tabs 0 Screens History)) (across 0.720 (down 0.780 View (tabs 0 Problems Messages)) Inspector))",
    ),
    (
        "Tall",
        "(across 0.760 (down 0.780 View (tabs 0 Problems Messages)) (down 0.450 (tabs 0 Scene Screens History) Inspector))",
    ),
    (
        "Wide view",
        "(across 0.150 (down 0.500 (tabs 0 Scene) (tabs 0 Screens History)) (across 0.800 (down 0.900 View (tabs 0 Problems Messages)) Inspector))",
    ),
];
pub(crate) const LAYOUTS_FILE: &str = "editor/layouts.txt";
pub(crate) const LAYOUT_UNREAD: &str =
    "the layout could not be read, so the panes stay as they are";
pub(crate) const CLOSE_PANE: &str = "Close";
pub(crate) const CLOSE_PANE_TIP: &str = "Close this pane, the window knob brings it back";
pub(crate) const MAXIMIZE_PANE: &str = "Maximize";
pub(crate) const MAXIMIZE_PANE_TIP: &str = "Fill the window with this pane";
pub(crate) const RESTORE_PANE: &str = "Restore";
pub(crate) const RESTORE_PANE_TIP: &str = "Put the layout back around this pane";
pub(crate) const RESET_LAYOUT: &str = "Reset to default";
pub(crate) const WINDOW_TIP: &str = "Layout: switch, save or delete a layout, or show a pane again";
pub(crate) const RESET_LAYOUT_TIP: &str = "Put every pane back where it started";
pub(crate) const SAVE_LAYOUT: &str = "Save layout as...";
pub(crate) const SAVE_LAYOUT_TIP: &str = "Keep this arrangement under a name, for every project";
pub(crate) const DELETE_LAYOUT: &str = "Delete layout...";
pub(crate) const DELETE_LAYOUT_TIP: &str = "Pick a saved layout to delete";
pub(crate) const WINDOW_PREFIX: &str = "Window: ";
pub(crate) const STORY_TIPS: [&str; 3] = [
    "Undo back to this step",
    "Where you are now",
    "Redo up to this step",
];
pub(crate) const SAVED_TIP: &str = "The last save";
