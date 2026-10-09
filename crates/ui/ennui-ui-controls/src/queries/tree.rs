use crate::components::Marks;

pub(crate) fn glyph_of(marks: Marks, open: bool) -> char {
    match open {
        true => marks.open,
        false => marks.shut,
    }
}
