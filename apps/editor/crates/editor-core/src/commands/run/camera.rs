use crate::data::{Context, Wait};
use crate::queries::tokens::{number, word};
use crate::theme::PICTURE_DELAY;
use editor_document::prelude::WORK;
use ennui::reflect::data::Token;

pub(crate) fn picture(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let name = line.get(1).map(|_| word(line, 1, "a name")).transpose()?;
    let frame = context.reach.time.frame;
    let name = name.unwrap_or_else(|| format!("picture-{frame}"));
    let given = std::path::Path::new(&name);
    let path = match given.is_absolute() {
        true => given.with_extension("png"),
        false => context
            .editor
            .book
            .root
            .join(WORK)
            .join(format!("pictures/{name}.png")),
    };
    if let Some(folder) = path.parent() {
        let _ = std::fs::create_dir_all(folder);
    }
    let wanted = frame + PICTURE_DELAY;
    context.editor.picture = Some((path.clone(), wanted));
    *context.wait = Some(Wait::Picture {
        path,
        frame: wanted,
    });
    Ok(())
}

pub(crate) fn wait(context: &mut Context, line: &[Token]) -> Result<(), String> {
    let frames = match line.get(1) {
        Some(_) => number(line, 1, "frames")? as u64,
        None => 2,
    };
    *context.wait = Some(Wait::Frames(context.reach.time.frame + frames));
    Ok(())
}
