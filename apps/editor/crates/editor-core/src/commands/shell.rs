use crate::commands::ask::ask;
use crate::commands::dock::default_tiles;
use crate::components::Viewing;
use crate::data::{Command, Follow, Logged, Problem, Watched};
use crate::queries::keys::keys_of;
use crate::queries::scenes::build_mark;
use crate::resources::{Editor, Shell};
use crate::theme::{
    CONTEXT_AWAY, FOOT, HINT, LAYOUTS, LINE_GAP, LISTS, LOG_MOST, SCENES_TIP, SEARCH_TIP,
    SHELL_ORDER, TOLD_LIFE, TOP, UNSAVED, WINDOW_TIP, WINDOW_TITLE,
};
use editor_document::prelude::{Author, scene_name, unsaved};
use ennui::later::{change, set, set_if_new, within};
use ennui::prelude::{Edits, Entity, Later, Storage};
use ennui::storage::{get as component, query};
use ennui_scene::prelude::despawn_trees;
use ennui_text::prelude::write_label;
use ennui_ui::prelude::{
    Dye, Float, Frame, Hidden, Lit, Rect, Span, Theme, afloat, chrome, frame, ink, label, panel,
    relay, scroll, touched, wrapped,
};

use ennui_ui_controls::prelude::{
    Band, Pick, Tray, dim, filling, foot_bar, hint, knob, menu, open_menu, rule, spread, tooltip,
    top_bar, tray_in, wording,
};
use ennui_ui_dock::prelude::{Board, board};
use ennui_ui_icons::prelude::{icon, icons};
use ennui_ui_rate::prelude::{Rate, lay_rate};
use nalgebra_glm::Vec2;

pub(crate) fn lay_shell(
    later: &mut Later,
    look: &Theme,
    shell: &mut Shell,
    (rate, tray): (&mut Rate, &mut Tray),
) {
    let root = chrome(later, look, SHELL_ORDER);
    relay(later, root, move |panel| panel.gap = 0.0);
    let top = top_bar(later, look, root, TOP);
    shell.top = Some(top);
    icon(later, look, top, icons::BOX, look.heading);
    label(
        later,
        look,
        top,
        &format!("Editor{}", build_mark()),
        look.heading,
    );
    rule(later, look, top);
    shell.scenes = Some(knob(later, look, top, icons::FOLDER_OPEN).0);
    shell.scene = Some(label(later, look, top, "", look.text));
    let state = label(later, look, top, "", look.text);
    shell.state = Some(dim(later, state));
    spread(later, look, top);
    shell.messages = Some(knob(later, look, top, icons::INFO).0);
    let (problems, shown) = knob(later, look, top, icons::TRIANGLE_ALERT);
    shell.problems = Some(problems);
    ink(later, shown, Dye::Faint, Dye::Bad);
    shell.problems_icon = Some(shown);
    let count = label(later, look, top, "", look.text);
    ink(later, count, Dye::Bad, Dye::Bad);
    shell.problems_count = Some(count);
    rule(later, look, top);
    let (undo, shown) = knob(later, look, top, icons::ARROW_LEFT);
    shell.undo = Some(undo);
    shell.undo_icon = Some(dim(later, shown));
    let (redo, shown) = knob(later, look, top, icons::ARROW_RIGHT);
    shell.redo = Some(redo);
    shell.redo_icon = Some(dim(later, shown));
    shell.save = Some(knob(later, look, top, icons::FILE).0);
    shell.searching = Some(knob(later, look, top, icons::SEARCH).0);
    let picker = panel(
        later,
        top,
        Frame::row(look)
            .wide(Span::Hug)
            .tall(Span::Fixed(look.row))
            .roles(Dye::Ground, Dye::Accent)
            .border(look.line)
            .rim(Dye::Edge)
            .pad(look.pad * 0.5)
            .gap(look.gap * 0.5),
    );
    icon(later, look, picker, icons::LAYOUT_GRID, look.text);
    shell.layout_label = Some(label(later, look, picker, LAYOUTS[0].0, look.text));
    shell.window = Some(touched(later, picker));
    shell.help = Some(knob(later, look, top, icons::CIRCLE_HELP).0);
    let board = board(later, look, root);
    shell.board = Some(board);
    let scene = panel(later, board, filling(look).gap(0.0));
    shell.scene_pane = Some(scene);
    let story = pane_body(later, look, board);
    shell.story = Some(scroll(later, look, story, pane_list(look)));
    let view = panel(later, board, filling(look).gap(0.0));
    set(later, view, Viewing);
    let pins = panel(later, view, filling(look).gap(0.0));
    relay(later, pins, |panel| panel.clips = true);
    shell.pin_sheet = Some(pins);
    tray.held = Some(tray_in(later, look, pins));
    let desk = panel(later, board, filling(look).gap(0.0));
    shell.inspector_pane = Some(desk);
    let logs = [0, 1].map(|kind| {
        let held = pane_body(later, look, board);
        let bar = panel(later, held, Frame::row(look).bare().pad(0.0));
        spread(later, look, bar);
        shell.log_copies[kind] = Some(knob(later, look, bar, icons::FILE).0);
        shell.log_shelves[kind] = Some(scroll(later, look, held, pane_list(look)));
        held
    });
    shell.panes = vec![scene, story, view, desk, logs[0], logs[1]];
    let texts = pane_body(later, look, board);
    let header = panel(later, texts, Frame::column(look).bare().pad(0.0));
    let listed = scroll(later, look, texts, pane_list(look));
    shell.text_pane = Some([texts, header, listed]);
    shell.panes.push(texts);
    let timeline = pane_body(later, look, board);
    let header = panel(later, timeline, Frame::column(look).bare().pad(0.0));
    let listed = scroll(later, look, timeline, pane_list(look));
    shell.timeline_pane = Some([timeline, header, listed]);
    shell.panes.push(timeline);
    let screens = pane_body(later, look, board);
    shell.screens_pane = Some(scroll(later, look, screens, pane_list(look)));
    shell.panes.push(screens);
    for held in &shell.panes {
        afloat(later, board, *held);
    }
    set(later, board, Board(default_tiles(&shell.panes)));
    shell.hint = Some(foot_bar(later, look, root, FOOT));
    lay_rate(later, look, rate, root);
    let lists = chrome(later, look, LISTS);
    shell.lists = Some(lists);
    for (held, said) in [
        (shell.scenes, SCENES_TIP),
        (shell.problems, "Problems: errors and warnings"),
        (
            shell.messages,
            "Messages: what Claude, the app and the editor said",
        ),
        (shell.window, WINDOW_TIP),
        (shell.searching, SEARCH_TIP),
        (shell.log_copies[0], "Copy the problems"),
        (shell.log_copies[1], "Copy the messages"),
    ] {
        if let Some(held) = held {
            tooltip(later, look, lists, held, said);
        }
    }
    for (held, said, action) in [
        (shell.save, "Save the layers", Command::Save),
        (shell.help, "Every command and its key", Command::Palette),
    ] {
        if let Some(held) = held {
            let said = format!("{said}   {}", keys_of(action));
            tooltip(later, look, lists, held, &said);
        }
    }
    shell.undo_tip = Some(tooltip(later, look, lists, undo, ""));
    shell.redo_tip = Some(tooltip(later, look, lists, redo, ""));
    let anchor = frame(
        later,
        Frame::new(look)
            .wide(Span::Fixed(0.0))
            .tall(Span::Fixed(0.0))
            .bare()
            .pad(0.0)
            .gap(0.0),
    );
    afloat(later, lists, anchor);
    set(later, anchor, Float(Vec2::new(CONTEXT_AWAY, CONTEXT_AWAY)));
    shell.context = Some(anchor);
    shell.laid = true;
}

fn pane_body(later: &mut Later, look: &Theme, board: Entity) -> Entity {
    panel(
        later,
        board,
        Frame::column(look)
            .role(Dye::Panel)
            .border(look.line)
            .rim(Dye::Edge)
            .round(0.0)
            .pad(look.pad)
            .gap(look.gap * 0.5),
    )
}

fn pane_list(look: &Theme) -> Frame<'_> {
    Frame::column(look)
        .tall(Span::Fill(1.0))
        .bare()
        .pad(0.0)
        .gap(LINE_GAP)
}
pub(crate) fn open_list(
    (seen, later): (&Storage, &mut Later),
    look: &Theme,
    (lists, knob): (Entity, Entity),
    options: &[&str],
) {
    let Some(rect) = component::<Rect>(seen, knob).copied() else {
        return;
    };
    menu(later, look, lists, knob, options);
    open_menu(later, knob, rect.center - rect.size * 0.5);
}

pub(crate) fn hint_rows(later: &mut Later, target: Entity, tips: Vec<String>) {
    change(later, move |storage| {
        let rows: Vec<(Entity, usize)> = query::<(&Pick, &Band)>(&*storage)
            .filter(|(_, (_, band))| band.0 == target)
            .map(|(row, (pick, _))| (row, pick.0))
            .collect();
        within(storage, |later| {
            for (row, place) in rows {
                if let Some(said) = tips.get(place) {
                    hint(later, row, said);
                }
            }
        });
    });
}

pub(crate) fn leave_for(later: &mut Later, shell: &mut Shell, editor: &mut Editor, scene: &str) {
    if scene == scene_name(&editor.book) {
        return;
    }
    let line = format!("open {scene}");
    let leaving = unsaved(&editor.book);
    match (leaving, shell.leave) {
        (true, Some(over)) => {
            set(later, over, Hidden(false));
            shell.leaving = Some(line);
        }
        _ => ask(editor, &line, &[&line], Follow::Tell),
    }
    shell.inspected = None;
}

pub fn fresh_list(
    later: &mut Later,
    look: &Theme,
    shelf: Entity,
    held: &mut Option<Entity>,
) -> Entity {
    if let Some(old) = held.take() {
        despawn_trees(later, vec![old]);
    }
    let list = panel(
        later,
        shelf,
        Frame::column(look).bare().pad(0.0).gap(LINE_GAP),
    );
    *held = Some(list);
    list
}

pub(crate) fn refresh<T: PartialEq>(
    watched: &mut Watched<T>,
    key: u64,
    read: impl FnOnce() -> T,
) -> bool {
    let first = !watched.looked;
    if watched.looked && watched.key == key {
        return false;
    }
    watched.key = key;
    watched.looked = true;
    let found = read();
    let changed = first || watched.held != found;
    watched.held = found;
    changed
}

pub fn report(editor: &mut Editor, source: &'static str, texts: Vec<String>) {
    let mut found: Vec<Problem> = Vec::new();
    for text in texts {
        let problem = Problem { source, text };
        if !found.contains(&problem) {
            found.push(problem);
        }
    }
    let (held, others): (Vec<Problem>, Vec<Problem>) = std::mem::take(&mut editor.standing)
        .into_iter()
        .partition(|problem| problem.source == source);
    let gone: Vec<Problem> = held
        .iter()
        .filter(|problem| !found.contains(problem))
        .cloned()
        .collect();
    editor.raised.retain(|problem| !gone.contains(problem));
    editor.withdrawn.extend(gone);
    let fresh = found.iter().filter(|problem| !held.contains(problem));
    editor.raised.extend(fresh.cloned());
    editor.standing = others;
    editor.standing.extend(found);
}

pub(crate) fn gather_words(
    shell: &mut Shell,
    editor: &mut Editor,
    problems: Vec<String>,
    step: f32,
) {
    let mut heard: Vec<(String, bool, Option<&'static str>)> = std::mem::take(&mut editor.said)
        .into_iter()
        .map(|said| (format!("Claude: {said}"), false, None))
        .collect();
    heard.extend(
        std::mem::take(&mut editor.notices)
            .into_iter()
            .chain(std::mem::take(&mut editor.told))
            .map(|said| (said, false, None)),
    );
    editor.problems.extend(problems);
    let mut booked = std::mem::take(&mut editor.book.problems);
    editor.problems.append(&mut booked);
    let withdrawn: Vec<(String, Option<&'static str>)> = std::mem::take(&mut editor.withdrawn)
        .into_iter()
        .map(|problem| (format!("problem: {}", problem.text), Some(problem.source)))
        .collect();
    let gone = |text: &String, source: Option<&'static str>| {
        source.is_some() && withdrawn.contains(&(text.clone(), source))
    };
    let unseen = shell.logged.iter().rev().filter(|logged| logged.problem);
    shell.unseen -= unseen
        .take(shell.unseen)
        .filter(|logged| gone(&logged.text, logged.source))
        .count();
    shell
        .logged
        .retain(|logged| !logged.problem || !gone(&logged.text, logged.source));
    shell
        .told
        .retain(|(held, _, problem, source)| !*problem || !gone(held, *source));
    heard.extend(
        std::mem::take(&mut editor.problems)
            .into_iter()
            .map(|problem| (format!("problem: {problem}"), true, None)),
    );
    heard.extend(
        std::mem::take(&mut editor.raised)
            .into_iter()
            .map(|problem| {
                (
                    format!("problem: {}", problem.text),
                    true,
                    Some(problem.source),
                )
            }),
    );
    for (said, problem, source) in heard {
        let told = shell
            .told
            .iter_mut()
            .find(|(held, _, _, from)| *held == said && *from == source);
        match told {
            Some((_, life, _, _)) => *life = TOLD_LIFE,
            None => {
                shell.unseen += usize::from(problem);
                shell.logged.push(Logged {
                    text: said.clone(),
                    problem,
                    source,
                });
                shell.told.push((said, TOLD_LIFE, problem, source));
            }
        }
    }
    let extra = shell.logged.len().saturating_sub(LOG_MOST);
    shell.logged.drain(..extra);
    for (_, life, _, _) in &mut shell.told {
        *life -= step;
    }
    shell.told.retain(|(_, life, _, _)| *life > 0.0);
}

pub(crate) fn write_words(
    (seen, edits): (&Storage, &mut Edits),
    shell: &mut Shell,
    editor: &Editor,
    title: &mut String,
) {
    let others = shell
        .told
        .iter()
        .rev()
        .skip(1)
        .filter(|(_, _, problem, _)| *problem)
        .count();
    let (hint, problem) = match shell.told.last() {
        Some((said, _, problem, _)) if others > 0 => {
            (format!("{said}   +{others} more in the log"), *problem)
        }
        Some((said, _, problem, _)) => (said.clone(), *problem),
        None => (HINT.to_string(), false),
    };
    if hint != shell.said {
        shell.said.clone_from(&hint);
        write_label(edits, shell.hint, hint);
        let ink = match problem {
            true => (Dye::Bad, Dye::Bad),
            false => (Dye::Faint, Dye::Ink),
        };
        if let Some(held) = shell.hint {
            set(edits, held, ink);
        }
    }
    let dirty = unsaved(&editor.book);
    let marked = match dirty {
        true => UNSAVED,
        false => "",
    };
    let scene = scene_name(&editor.book);
    write_label(edits, shell.scene, format!("{scene}{marked}"));
    let wanted = format!("{WINDOW_TITLE} - {scene}{marked}{}", build_mark());
    if *title != wanted {
        *title = wanted;
    }
    let state = match dirty {
        true => String::from("Not saved"),
        false => String::from("Saved"),
    };
    write_label(edits, shell.state, state);
    let count = match shell.unseen {
        0 => String::new(),
        count => count.to_string(),
    };
    write_label(edits, shell.problems_count, count);
    if let Some(shown) = shell.problems_icon {
        set_if_new(edits, shown, Lit(shell.unseen > 0));
    }
    let undo = match editor.book.done.last() {
        Some(change) => format!("Undo {} ({})", change.label, author_name(change.author)),
        None => String::from("Nothing to undo"),
    };
    let redo = match editor.book.undone.last() {
        Some(change) => format!("Redo {} ({})", change.label, author_name(change.author)),
        None => String::from("Nothing to redo"),
    };
    write_label(
        edits,
        shell.undo_tip.and_then(|card| wording(seen, card)),
        format!(
            "{undo}   {}, Ctrl click undoes everything",
            keys_of(Command::Undo)
        ),
    );
    write_label(
        edits,
        shell.redo_tip.and_then(|card| wording(seen, card)),
        format!("{redo}   {}", keys_of(Command::Redo)),
    );
    for (shown, on) in [
        (shell.undo_icon, !editor.book.done.is_empty()),
        (shell.redo_icon, !editor.book.undone.is_empty()),
    ] {
        if let Some(shown) = shown {
            set_if_new(edits, shown, Lit(on));
        }
    }
}

fn author_name(author: Author) -> &'static str {
    match author {
        Author::User => "you",
        Author::Claude => "Claude",
    }
}

pub(crate) fn write_log_lines(
    later: &mut Later,
    look: &Theme,
    list: Entity,
    lines: &[&str],
    problems: bool,
    (height, room): (f32, f32),
) {
    for text in lines {
        let said = wrapped(later, look, list, text, height, room);
        let ink = match problems {
            true => (Dye::Bad, Dye::Bad),
            false => (Dye::Ink, Dye::Ink),
        };
        set(later, said, ink);
    }
}
