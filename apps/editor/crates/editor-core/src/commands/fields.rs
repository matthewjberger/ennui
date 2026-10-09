use crate::data::{Control, Held, Laying, Step};
use crate::queries::describe::kind_text;
use crate::queries::inspect::{
    color_of, default_of, kind_of_value, numbers_of, shape_of, spoken, step_of, value_at,
};
use crate::theme::{LABEL, LIST_MOST, NEST, PICK_FILE_TIP, TEXT_ROOM, WIDE_RANGE};
use ennui::later::change;
use ennui::prelude::{Edits, Entity, Later};
use ennui::reflect::prelude::{Field, Kind, Value, written};
use ennui::storage::{get, set_if_new};
use ennui_text::prelude::Label;
use ennui_ui::prelude::{Dye, Frame, Span, Theme, panel, spacing};

use ennui_ui_controls::prelude::{
    Knob, MIXED, dim, entry, field as text_field, hint, knob, ledger, listed, noted, put_dropdown,
    scrub, small, swatch, toggle, tooltip,
};
use ennui_ui_icons::prelude::icons;

type Shape<'held> = (&'held Kind, &'held [Field], Option<&'held Field>);

fn control_of(laying: &Laying, steps: &[Step], parts: Vec<Entity>, held: Held) -> Control {
    let seen: Vec<Option<&Value>> = laying
        .values
        .iter()
        .map(|value| value_at(value, steps))
        .collect();
    Control {
        ids: laying.ids.to_vec(),
        component: String::from(laying.component),
        steps: steps.to_vec(),
        parts,
        shape: shape_of(&held, &seen),
        held,
        picks: Vec::new(),
        setting: laying.setting,
    }
}

fn mixed_scrub(edits: &mut Edits, part: Entity) {
    change(edits, move |storage| {
        if let Some(shown) = get::<Knob>(&*storage, part).map(|held| held.0) {
            set_if_new(storage, shown, Label(String::from(MIXED)));
        }
    });
}

fn joined(steps: &[Step], step: Step) -> Vec<Step> {
    let mut held = steps.to_vec();
    held.push(step);
    held
}

pub(crate) fn lay_pairs(
    later: &mut Later,
    laying: &Laying,
    controls: &mut Vec<Control>,
    (form, steps): (Entity, &[Step]),
    pairs: &[(String, Value)],
    fields: &[Field],
) {
    let mut names: Vec<&str> = Vec::new();
    let described = fields.iter().map(|field| field.name);
    for name in described.chain(pairs.iter().map(|(key, _)| key.as_str())) {
        if !names.contains(&name) && pairs.iter().any(|(key, _)| key == name) {
            names.push(name);
        }
    }
    for name in names {
        let field = fields.iter().find(|field| field.name == name);
        let inner = field.map(|field| (field.fields)()).unwrap_or_default();
        let kind = field.map_or(Kind::Other, |field| field.kind.clone());
        let label = spoken(name);
        let path = joined(steps, Step::Key(String::from(name)));
        lay_field(
            later,
            laying,
            controls,
            (form, &path),
            (&kind, &inner, field),
            &label,
        );
    }
}

pub(crate) fn lay_field(
    later: &mut Later,
    laying: &Laying,
    controls: &mut Vec<Control>,
    (form, steps): (Entity, &[Step]),
    shape: Shape,
    label: &str,
) {
    let line = entry(later, laying.look, form, label);
    let path: Vec<String> = steps
        .iter()
        .filter_map(|step| match step {
            Step::Key(key) => Some(key.clone()),
            Step::Item(place) => Some(place.to_string()),
            Step::Inner => None,
        })
        .collect();
    let range = shape
        .2
        .and_then(|field| field.range)
        .map(|(low, high)| format!(", from {low} to {high}"))
        .unwrap_or_default();
    let about = shape
        .2
        .filter(|field| !field.about.is_empty())
        .map(|field| format!(". {}", field.about))
        .unwrap_or_default();
    let tip = format!(
        "{}.{}: {}{range}{about}",
        laying.component,
        path.join("."),
        kind_text(shape.0)
    );
    noted(later, laying.look, laying.lists, line, &tip);
    let before = controls.len();
    lay_value(later, laying, controls, (form, line), steps, shape);
    for control in controls[before..]
        .iter()
        .filter(|control| control.steps == steps)
    {
        let parts = match control.held {
            Held::Items(..) => &[][..],
            Held::Path(_) => &control.parts[..1],
            _ => &control.parts[..],
        };
        for part in parts {
            hint(later, *part, &tip);
        }
    }
}

fn nested_of(later: &mut Later, look: &Theme, form: Entity) -> Entity {
    let row = panel(later, form, Frame::row(look).bare().pad(0.0).gap(0.0));
    spacing(later, look, row, NEST);
    panel(
        later,
        row,
        Frame::new(look)
            .wide(Span::Fixed(look.line))
            .tall(Span::Fill(1.0))
            .role(Dye::Edge)
            .round(0.0)
            .pad(0.0)
            .gap(0.0),
    );
    ledger(later, look, row, LABEL)
}

fn lay_inside(
    later: &mut Later,
    laying: &Laying,
    controls: &mut Vec<Control>,
    (form, steps): (Entity, &[Step]),
    value: &Value,
    fields: &[Field],
) {
    let nested = nested_of(later, laying.look, form);
    match value {
        Value::Record(pairs) => lay_pairs(later, laying, controls, (nested, steps), pairs, fields),
        _ => lay_field(
            later,
            laying,
            controls,
            (nested, steps),
            (&Kind::Other, fields, None),
            "Value",
        ),
    }
}

fn lay_value(
    later: &mut Later,
    laying: &Laying,
    controls: &mut Vec<Control>,
    (form, line): (Entity, Entity),
    steps: &[Step],
    (kind, fields, field): Shape,
) {
    let look = laying.look;
    let seen: Vec<Option<&Value>> = laying
        .values
        .iter()
        .map(|value| value_at(value, steps))
        .collect();
    let Some(value) = seen.first().copied().flatten().cloned() else {
        let said = small(later, look, line, MIXED);
        dim(later, said);
        return;
    };
    let mixed = seen.iter().any(|held| *held != Some(&value));
    let kind = match kind {
        Kind::Other | Kind::List => kind_of_value(&value),
        other => other.clone(),
    };
    let name = steps
        .iter()
        .rev()
        .find_map(|step| match step {
            Step::Key(key) => Some(key.as_str()),
            _ => None,
        })
        .unwrap_or_default();
    let numbered = numbers_of(&value).is_some_and(|numbers| (1..=4).contains(&numbers.len()));
    let held = match (&kind, &value) {
        (Kind::Optional(inner), _) => {
            lay_optional(
                later,
                laying,
                controls,
                (form, line, steps),
                (inner, fields, field),
            );
            return;
        }
        (Kind::Choice(names), Value::Word(_) | Value::Variant(..)) => {
            lay_choice(later, laying, controls, (form, line, steps), names, fields);
            return;
        }
        (Kind::Color, _) if color_of(&value).is_some() => {
            let (color, count) = color_of(&value).unwrap_or_default();
            let held = swatch(later, look, line, laying.lists, color);
            if mixed {
                let said = small(later, look, line, MIXED);
                dim(later, said);
            }
            (vec![held], Held::Tint(color, count))
        }
        (_, Value::List(_)) if numbered => {
            let numbers = numbers_of(&value).unwrap_or_default();
            let step = step_of(&kind, field, name, laying.cell);
            let mut parts = Vec::new();
            for (place, number) in numbers.iter().enumerate() {
                let part = scrub(later, look, line, *number, step, (-WIDE_RANGE, WIDE_RANGE));
                let split = seen.iter().any(|held| {
                    held.and_then(numbers_of)
                        .and_then(|numbers| numbers.get(place).copied())
                        != Some(*number)
                });
                if split {
                    mixed_scrub(later, part);
                }
                parts.push(part);
            }
            (parts, Held::Numbers(numbers))
        }
        (_, Value::Number(number)) => {
            let step = step_of(&kind, field, name, laying.cell);
            let range = field
                .and_then(|field| field.range)
                .map_or((-WIDE_RANGE, WIDE_RANGE), |(low, high)| {
                    (low as f32, high as f32)
                });
            let held = scrub(later, look, line, *number as f32, step, range);
            if mixed {
                mixed_scrub(later, held);
            }
            (vec![held], Held::Number(*number as f32))
        }
        (_, Value::Bool(flag)) => {
            let said = if mixed { MIXED } else { "" };
            let held = toggle(later, look, line, said, *flag && !mixed);
            (vec![held], Held::Flag(*flag && !mixed))
        }
        (Kind::Path(extensions), Value::Text(text)) => {
            lay_path(later, laying, controls, (line, steps), text, extensions);
            return;
        }
        (_, Value::Text(text) | Value::Reference(text)) => {
            let (shown, prompt) = if mixed {
                ("", MIXED)
            } else {
                (text.as_str(), "")
            };
            let held = text_field(later, look, line, shown, prompt, TEXT_ROOM);
            let kept = String::from(shown);
            match value {
                Value::Reference(_) => (vec![held], Held::Named(kept)),
                _ => (vec![held], Held::Words(kept)),
            }
        }
        (_, Value::Record(_)) => {
            lay_inside(later, laying, controls, (form, steps), &value, fields);
            return;
        }
        (_, Value::List(items)) => {
            let item = field.and_then(|field| (field.item)());
            lay_list(
                later,
                laying,
                controls,
                (form, line, steps),
                items,
                (fields, item),
            );
            return;
        }
        (_, Value::Variant(named, inner)) => {
            let said = small(later, look, line, named);
            dim(later, said);
            let steps = joined(steps, Step::Inner);
            lay_inside(later, laying, controls, (form, &steps), inner, fields);
            return;
        }
        _ => {
            let said = small(later, look, line, &written(&value));
            dim(later, said);
            return;
        }
    };
    controls.push(control_of(laying, steps, held.0, held.1));
}

fn lay_optional(
    later: &mut Later,
    laying: &Laying,
    controls: &mut Vec<Control>,
    (form, line, steps): (Entity, Entity, &[Step]),
    (inner, fields, field): Shape,
) {
    let none = |held: &Value| matches!(held, Value::Word(word) if word == "none");
    let seen: Vec<Option<&Value>> = laying
        .values
        .iter()
        .map(|value| value_at(value, steps))
        .collect();
    let present = seen
        .first()
        .copied()
        .flatten()
        .is_some_and(|held| !none(held));
    let split = seen
        .iter()
        .any(|held| held.is_none_or(|held| none(held) == present));
    let said = if split { MIXED } else { "" };
    let on = present && !split;
    let held = toggle(later, laying.look, line, said, on);
    let made = default_of(inner);
    controls.push(control_of(
        laying,
        steps,
        vec![held],
        Held::Present(on, made),
    ));
    if on {
        lay_value(
            later,
            laying,
            controls,
            (form, line),
            steps,
            (inner, fields, field),
        );
    }
}

fn lay_choice(
    later: &mut Later,
    laying: &Laying,
    controls: &mut Vec<Control>,
    (form, line, steps): (Entity, Entity, &[Step]),
    names: &[&str],
    fields: &[Field],
) {
    let named = |held: &Value| match held {
        Value::Word(word) | Value::Variant(word, _) => Some(word.clone()),
        _ => None,
    };
    let seen: Vec<Option<&Value>> = laying
        .values
        .iter()
        .map(|value| value_at(value, steps))
        .collect();
    let Some(value) = seen.first().copied().flatten() else {
        return;
    };
    let word = named(value);
    let at = names
        .iter()
        .position(|name| Some(*name) == word.as_deref())
        .unwrap_or(0);
    let held = listed(later, laying.look, [line, laying.lists], names, at, false);
    let split = seen.iter().any(|held| held.and_then(named) != word);
    if split {
        put_dropdown(later, held, None);
    }
    let owned = names.iter().map(|name| String::from(*name)).collect();
    controls.push(control_of(laying, steps, vec![held], Held::Pick(at, owned)));
    if let (false, Value::Variant(_, inner)) = (split, value)
        && **inner != Value::Unit
    {
        let steps = joined(steps, Step::Inner);
        lay_inside(later, laying, controls, (form, &steps), inner, fields);
    }
}

fn lay_path(
    later: &mut Later,
    laying: &Laying,
    controls: &mut Vec<Control>,
    (line, steps): (Entity, &[Step]),
    text: &str,
    extensions: &[&str],
) {
    let picks = laying
        .files
        .iter()
        .filter(|file| {
            let lower = file.to_lowercase();
            extensions
                .iter()
                .any(|extension| lower.ends_with(&format!(".{extension}")))
        })
        .cloned()
        .collect();
    lay_offer(later, laying, controls, (line, steps), text, picks);
}

fn lay_offer(
    later: &mut Later,
    laying: &Laying,
    controls: &mut Vec<Control>,
    (line, steps): (Entity, &[Step]),
    text: &str,
    picks: Vec<String>,
) {
    let look = laying.look;
    let split = laying
        .values
        .iter()
        .any(|value| value_at(value, steps) != Some(&Value::Text(String::from(text))));
    let (shown, prompt) = if split { ("", MIXED) } else { (text, "") };
    let held = text_field(later, look, line, shown, prompt, TEXT_ROOM);
    let pick = knob(later, look, line, icons::FOLDER_OPEN).0;
    tooltip(later, look, laying.lists, pick, PICK_FILE_TIP);
    hint(later, pick, PICK_FILE_TIP);
    controls.push(Control {
        picks,
        ..control_of(
            laying,
            steps,
            vec![held, pick],
            Held::Path(String::from(shown)),
        )
    });
}

fn lay_list(
    later: &mut Later,
    laying: &Laying,
    controls: &mut Vec<Control>,
    (form, line, steps): (Entity, Entity, &[Step]),
    items: &[Value],
    (fields, item): (&[Field], Option<Value>),
) {
    let look = laying.look;
    let said = small(later, look, line, &format!("{} items", items.len()));
    dim(later, said);
    let split = laying.values.iter().any(|value| {
        !matches!(value_at(value, steps), Some(Value::List(held)) if held.len() == items.len())
    });
    if split || (items.is_empty() && item.is_none()) {
        return;
    }
    let more = knob(later, look, line, icons::PLUS).0;
    let less = knob(later, look, line, icons::MINUS).0;
    let adds = match items.is_empty() {
        true => "Add an item with default values",
        false => "Add a copy of the last item",
    };
    tooltip(later, look, laying.lists, more, adds);
    tooltip(later, look, laying.lists, less, "Remove the last item");
    hint(later, more, adds);
    hint(later, less, "Remove the last item");
    controls.push(control_of(
        laying,
        steps,
        vec![more, less],
        Held::Items(items.len(), item),
    ));
    let nested = nested_of(later, look, form);
    for place in 0..items.len().min(LIST_MOST) {
        let path = joined(steps, Step::Item(place));
        lay_field(
            later,
            laying,
            controls,
            (nested, &path),
            (&Kind::Other, fields, None),
            &place.to_string(),
        );
    }
    if items.len() > LIST_MOST {
        let said = small(
            later,
            look,
            nested,
            &format!("{} more", items.len() - LIST_MOST),
        );
        dim(later, said);
    }
}
