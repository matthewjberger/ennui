use crate::commands::dock::write_layouts;
use crate::data::Context;
use crate::queries::layouts::{layout_names, layout_of};
use crate::queries::tokens::{ids_from, word};
use crate::theme::{LAYOUTS, PANES};
use ennui::reflect::data::Token;

pub(crate) fn layout(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let editor = &mut *context.editor;
    if line.len() < 2 {
        context
            .reply
            .push(format!("layouts: {}", layout_names(editor).join(", ")));
        context
            .reply
            .push(format!("current: {}", editor.layout_name));
        return Ok(());
    }
    let verb = word(line, 1, "use, save, delete, reset or show")?;
    let name = match verb.as_str() {
        "reset" => String::from(LAYOUTS[0].0),
        _ => ids_from(line, 2)?.join(" "),
    };
    if name.is_empty() {
        return Err(format!("layout {verb} wants a layout name"));
    }
    let built = LAYOUTS.iter().any(|(held, _)| *held == name);
    match verb.as_str() {
        "show" => {
            let place = PANES
                .iter()
                .position(|(pane, _)| pane.eq_ignore_ascii_case(&name))
                .ok_or_else(|| {
                    let names: Vec<&str> = PANES.iter().map(|(pane, _)| *pane).collect();
                    format!("{name} is not a pane; they are {}", names.join(", "))
                })?;
            editor.pane_asked = Some(place);
            context
                .reply
                .push(format!("the {} pane comes to the front", PANES[place].0));
        }
        "reset" | "use" => {
            if layout_of(editor, &name).is_none() {
                return Err(format!("{name} is not a layout; layout lists them"));
            }
            editor.layout_asked = Some(name.clone());
            context
                .reply
                .push(format!("the panes take the {name} layout"));
        }
        "save" => {
            if built {
                return Err(format!("{name} is a built-in layout; pick another name"));
            }
            let text = editor.layout_text.clone();
            match editor.layouts.iter_mut().find(|(held, _)| *held == name) {
                Some((_, held)) => *held = text,
                None => editor.layouts.push((name.clone(), text)),
            }
            write_layouts(&editor.layouts)?;
            editor.layout_name = name.clone();
            context.reply.push(format!("saved the layout {name}"));
        }
        "delete" => {
            let before = editor.layouts.len();
            editor.layouts.retain(|(held, _)| *held != name);
            if editor.layouts.len() == before {
                return Err(format!("{name} is not a saved layout"));
            }
            write_layouts(&editor.layouts)?;
            if editor.layout_name == name {
                editor.layout_asked = Some(String::from(LAYOUTS[0].0));
            }
            context.reply.push(format!("deleted the layout {name}"));
        }
        other => {
            return Err(format!(
                "layout wants use, save, delete, reset or show, not {other}"
            ));
        }
    }
    Ok(())
}
