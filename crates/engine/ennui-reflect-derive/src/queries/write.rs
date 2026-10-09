use crate::data::{Body, FIELD, Item, KIND, Notes, REFLECT, SAVE, Shape, Slot, VALUE, Variant};

fn kept(slots: &[Slot]) -> Vec<(usize, &Slot)> {
    slots
        .iter()
        .enumerate()
        .filter(|(_, slot)| !slot.notes.skip)
        .collect()
}

fn kind_of(slot: &Slot) -> String {
    match (&slot.notes.path, slot.notes.color) {
        (Some(extensions), _) => format!("{KIND}::Path(::std::vec![{}])", extensions.join(", ")),
        (None, true) => format!("{KIND}::Color"),
        (None, false) => format!("<{} as {REFLECT}>::kind()", slot.kind),
    }
}

fn field_of(name: &str, slot: &Slot) -> String {
    let Notes {
        snapshot,
        range,
        step,
        about,
        ..
    } = &slot.notes;
    let save = if *snapshot { "Snapshot" } else { "Scene" };
    let range = match range {
        Some((low, high)) => format!("Some((({low}) as f64, ({high}) as f64))"),
        None => String::from("None"),
    };
    let step = match step {
        Some(step) => format!("Some(({step}) as f64)"),
        None => String::from("None"),
    };
    let about = about.as_deref().unwrap_or("\"\"");
    format!(
        "{FIELD} {{ name: \"{name}\", kind: {}, save: {SAVE}::{save}, range: {range}, step: {step}, about: {about}, fields: <{} as {REFLECT}>::fields, item: <{} as {REFLECT}>::item }}",
        kind_of(slot),
        slot.kind,
        slot.kind
    )
}

fn binding(place: usize, slot: &Slot) -> String {
    match &slot.name {
        Some(name) => name.clone(),
        None => format!("field_{place}"),
    }
}

fn keyed(slots: &[Slot], access: impl Fn(&str) -> String) -> Vec<(String, String, &Slot)> {
    kept(slots)
        .into_iter()
        .map(|(place, slot)| {
            let name = binding(place, slot);
            (
                name.strip_prefix("r#").unwrap_or(&name).to_string(),
                access(&name),
                slot,
            )
        })
        .collect()
}

fn record_value(pairs: &[(String, String, &Slot)]) -> String {
    let items: Vec<String> = pairs
        .iter()
        .map(|(key, access, slot)| {
            format!(
                "(::std::string::String::from(\"{key}\"), <{} as {REFLECT}>::value_of({access}))",
                slot.kind
            )
        })
        .collect();
    format!("{VALUE}::Record(::std::vec![{}])", items.join(", "))
}

fn list_value(pairs: &[(String, &Slot)]) -> String {
    let items: Vec<String> = pairs
        .iter()
        .map(|(access, slot)| format!("<{} as {REFLECT}>::value_of({access})", slot.kind))
        .collect();
    format!("{VALUE}::List(::std::vec![{}])", items.join(", "))
}

fn record_apply(pairs: &[(String, String, &Slot)], source: &str) -> String {
    if pairs.is_empty() {
        return format!("matches!({source}, {VALUE}::Record(_))");
    }
    let arms: Vec<String> = pairs
        .iter()
        .map(|(key, access, slot)| {
            format!(
                "if reflect_key == \"{key}\" {{ reflect_taken &= <{} as {REFLECT}>::apply({access}, reflect_inner); }}",
                slot.kind
            )
        })
        .collect();
    format!(
        "match {source} {{ {VALUE}::Record(reflect_pairs) => {{ let mut reflect_taken = true; for (reflect_key, reflect_inner) in reflect_pairs.iter() {{ {} else {{ reflect_taken = false; }} }} reflect_taken }} _ => false }}",
        arms.join(" else ")
    )
}

fn list_apply(pairs: &[(String, &Slot)], source: &str) -> String {
    if pairs.is_empty() {
        return format!("matches!({source}, {VALUE}::List(_))");
    }
    let steps: Vec<String> = pairs
        .iter()
        .enumerate()
        .map(|(place, (access, slot))| {
            format!(
                "if let Some(reflect_inner) = reflect_items.get({place}) {{ reflect_taken &= <{} as {REFLECT}>::apply({access}, reflect_inner); }}",
                slot.kind
            )
        })
        .collect();
    format!(
        "match {source} {{ {VALUE}::List(reflect_items) => {{ let mut reflect_taken = true; {} reflect_taken }} _ => false }}",
        steps.join(" ")
    )
}

struct Methods {
    value_of: String,
    apply: String,
    kind: String,
    fields: String,
    uses_held: bool,
}

fn struct_methods(shape: &Shape) -> Methods {
    match shape {
        Shape::Unit => Methods {
            value_of: format!("{VALUE}::Unit"),
            apply: String::from("true"),
            kind: format!("{KIND}::Other"),
            fields: String::from("::std::vec::Vec::new()"),
            uses_held: false,
        },
        Shape::Named(slots) => {
            let reads = keyed(slots, |name| format!("&reflect_held.{name}"));
            let writes = keyed(slots, |name| format!("&mut reflect_held.{name}"));
            let fields: Vec<String> = reads
                .iter()
                .map(|(key, _, slot)| field_of(key, slot))
                .collect();
            Methods {
                value_of: record_value(&reads),
                apply: record_apply(&writes, "value"),
                kind: format!("{KIND}::Record"),
                fields: format!("::std::vec![{}]", fields.join(", ")),
                uses_held: !reads.is_empty(),
            }
        }
        Shape::Tuple(slots) => {
            let kept = kept(slots);
            if kept.len() == 1 && slots.len() == 1 {
                let slot = kept[0].1;
                return Methods {
                    value_of: format!("<{} as {REFLECT}>::value_of(&reflect_held.0)", slot.kind),
                    apply: format!(
                        "<{} as {REFLECT}>::apply(&mut reflect_held.0, value)",
                        slot.kind
                    ),
                    kind: kind_of(slot),
                    fields: format!("<{} as {REFLECT}>::fields()", slot.kind),
                    uses_held: true,
                };
            }
            let reads: Vec<(String, &Slot)> = kept
                .iter()
                .map(|(place, slot)| (format!("&reflect_held.{place}"), *slot))
                .collect();
            let writes: Vec<(String, &Slot)> = kept
                .iter()
                .map(|(place, slot)| (format!("&mut reflect_held.{place}"), *slot))
                .collect();
            let fields: Vec<String> = kept
                .iter()
                .map(|(place, slot)| field_of(&place.to_string(), slot))
                .collect();
            Methods {
                value_of: list_value(&reads),
                apply: list_apply(&writes, "value"),
                kind: format!("{KIND}::List"),
                fields: format!("::std::vec![{}]", fields.join(", ")),
                uses_held: !kept.is_empty(),
            }
        }
    }
}

fn pattern_of(variant: &Variant) -> String {
    let name = &variant.name;
    match &variant.shape {
        Shape::Unit => format!("Self::{name}"),
        Shape::Named(slots) => {
            let bound: Vec<String> = kept(slots)
                .iter()
                .map(|(place, slot)| binding(*place, slot))
                .collect();
            if bound.is_empty() {
                format!("Self::{name} {{ .. }}")
            } else {
                format!("Self::{name} {{ {}, .. }}", bound.join(", "))
            }
        }
        Shape::Tuple(slots) => {
            let bound: Vec<String> = slots
                .iter()
                .enumerate()
                .map(|(place, slot)| {
                    if slot.notes.skip {
                        String::from("_")
                    } else {
                        binding(place, slot)
                    }
                })
                .collect();
            format!("Self::{name}({})", bound.join(", "))
        }
    }
}

fn shape_of(variant: &Variant) -> String {
    let name = &variant.name;
    match &variant.shape {
        Shape::Unit => format!("Self::{name}"),
        Shape::Named(_) => format!("Self::{name} {{ .. }}"),
        Shape::Tuple(_) => format!("Self::{name}(..)"),
    }
}

fn made_value(slot: &Slot) -> String {
    slot.notes
        .made
        .clone()
        .unwrap_or_else(|| String::from("::std::default::Default::default()"))
}

fn made_of(variant: &Variant) -> String {
    let name = &variant.name;
    match &variant.shape {
        Shape::Unit => format!("Self::{name}"),
        Shape::Named(slots) => {
            let parts: Vec<String> = slots
                .iter()
                .enumerate()
                .map(|(place, slot)| format!("{}: {}", binding(place, slot), made_value(slot)))
                .collect();
            format!("Self::{name} {{ {} }}", parts.join(", "))
        }
        Shape::Tuple(slots) => {
            let parts: Vec<String> = slots.iter().map(made_value).collect();
            format!("Self::{name}({})", parts.join(", "))
        }
    }
}

fn variant_value(variant: &Variant) -> String {
    let name = &variant.name;
    let inner = match &variant.shape {
        Shape::Unit => return format!("{VALUE}::Word(::std::string::String::from(\"{name}\"))"),
        Shape::Named(slots) => record_value(&keyed(slots, str::to_string)),
        Shape::Tuple(slots) => {
            let kept = kept(slots);
            if kept.len() == 1 && slots.len() == 1 {
                let slot = kept[0].1;
                format!(
                    "<{} as {REFLECT}>::value_of({})",
                    slot.kind,
                    binding(kept[0].0, slot)
                )
            } else {
                let pairs: Vec<(String, &Slot)> = kept
                    .iter()
                    .map(|(place, slot)| (binding(*place, slot), *slot))
                    .collect();
                list_value(&pairs)
            }
        }
    };
    format!(
        "{VALUE}::Variant(::std::string::String::from(\"{name}\"), ::std::boxed::Box::new({inner}))"
    )
}

fn variant_apply(variant: &Variant) -> String {
    match &variant.shape {
        Shape::Unit => String::from("true"),
        Shape::Named(slots) => record_apply(&keyed(slots, str::to_string), "reflect_inner"),
        Shape::Tuple(slots) => {
            let kept = kept(slots);
            if kept.len() == 1 && slots.len() == 1 {
                let slot = kept[0].1;
                format!(
                    "{{ let reflect_alone = match reflect_inner {{ {VALUE}::List(reflect_items) if reflect_items.len() == 1 => &reflect_items[0], reflect_other => reflect_other }}; <{kind} as {REFLECT}>::apply({bound}, reflect_inner) || <{kind} as {REFLECT}>::apply({bound}, reflect_alone) }}",
                    kind = slot.kind,
                    bound = binding(kept[0].0, slot)
                )
            } else {
                let pairs: Vec<(String, &Slot)> = kept
                    .iter()
                    .map(|(place, slot)| (binding(*place, slot), *slot))
                    .collect();
                list_apply(&pairs, "reflect_inner")
            }
        }
    }
}

fn variant_fields(variants: &[Variant]) -> String {
    let mut named: Vec<String> = Vec::new();
    let mut nested: Vec<String> = Vec::new();
    for variant in variants {
        match &variant.shape {
            Shape::Named(slots) => {
                for (key, _, slot) in keyed(slots, str::to_string) {
                    named.push(field_of(&key, slot));
                }
            }
            Shape::Tuple(slots) if slots.len() == 1 && !slots[0].notes.skip => {
                nested.push(format!("<{} as {REFLECT}>::fields()", slots[0].kind));
            }
            _ => {}
        }
    }
    let listed = format!("::std::vec![{}]", named.join(", "));
    if nested.is_empty() {
        return listed;
    }
    let extended: Vec<String> = nested
        .iter()
        .map(|inner| format!("reflect_fields.extend({inner});"))
        .collect();
    format!(
        "{{ let mut reflect_fields: ::std::vec::Vec<{FIELD}> = {listed}; {} reflect_fields }}",
        extended.join(" ")
    )
}

fn enum_methods(variants: &[Variant]) -> Methods {
    let reads: Vec<String> = variants
        .iter()
        .map(|variant| format!("{} => {}", pattern_of(variant), variant_value(variant)))
        .collect();
    let words: Vec<String> = variants
        .iter()
        .map(|variant| {
            format!(
                "if word.eq_ignore_ascii_case(\"{}\") {{ *reflect_held = {}; return true; }}",
                variant.name,
                made_of(variant)
            )
        })
        .collect();
    let shaped: Vec<String> = variants
        .iter()
        .filter(|variant| !matches!(variant.shape, Shape::Unit))
        .map(|variant| {
            format!(
                "if reflect_name == \"{name}\" {{ if !matches!(reflect_held, {shape}) {{ *reflect_held = {made}; }} if let {pattern} = reflect_held {{ return {apply}; }} return false; }}",
                name = variant.name,
                shape = shape_of(variant),
                pattern = pattern_of(variant),
                made = made_of(variant),
                apply = variant_apply(variant),
            )
        })
        .collect();
    let shaped_arm = if shaped.is_empty() {
        String::new()
    } else {
        format!(
            "{VALUE}::Variant(reflect_name, reflect_inner) => {{ let reflect_inner = &**reflect_inner; {} false }}",
            shaped.join(" ")
        )
    };
    let apply = format!(
        "match value {{ {VALUE}::Word(word) => {{ {} false }} {shaped_arm} _ => false }}",
        words.join(" ")
    );
    let names: Vec<String> = variants
        .iter()
        .map(|variant| format!("\"{}\"", variant.name))
        .collect();
    Methods {
        value_of: if variants.is_empty() {
            format!("{VALUE}::Unit")
        } else {
            format!("match reflect_held {{ {} }}", reads.join(", "))
        },
        apply,
        kind: format!("{KIND}::Choice(::std::vec![{}])", names.join(", ")),
        fields: variant_fields(variants),
        uses_held: !variants.is_empty(),
    }
}

fn slot_kinds(item: &Item) -> Vec<String> {
    let slots: Vec<&Slot> = match &item.body {
        Body::Struct(Shape::Named(slots)) | Body::Struct(Shape::Tuple(slots)) => {
            slots.iter().collect()
        }
        Body::Struct(Shape::Unit) => Vec::new(),
        Body::Enum(variants) => variants
            .iter()
            .flat_map(|variant| match &variant.shape {
                Shape::Named(slots) | Shape::Tuple(slots) => slots.iter().collect(),
                Shape::Unit => Vec::new(),
            })
            .collect(),
    };
    let mut kinds: Vec<String> = slots
        .iter()
        .filter(|slot| !slot.notes.skip)
        .map(|slot| slot.kind.clone())
        .collect();
    kinds.sort();
    kinds.dedup();
    kinds
}

pub(crate) fn written(item: &Item) -> String {
    let methods = match &item.body {
        Body::Struct(shape) => struct_methods(shape),
        Body::Enum(variants) => enum_methods(variants),
    };
    let mut predicates: Vec<String> = Vec::new();
    if !item.wheres.trim().is_empty() {
        predicates.push(item.wheres.trim().trim_end_matches(',').to_string());
    }
    if !item.params.is_empty() {
        predicates.extend(item.bounds.iter().cloned());
        let defaulted = matches!(item.body, Body::Enum(_));
        for kind in slot_kinds(item) {
            predicates.push(format!("{kind}: {REFLECT}"));
            if defaulted {
                predicates.push(format!("{kind}: ::std::default::Default"));
            }
        }
    }
    let wheres = if predicates.is_empty() {
        String::new()
    } else {
        format!("where {}", predicates.join(", "))
    };
    let held = if methods.uses_held {
        "reflect_held"
    } else {
        "_held"
    };
    let value = if methods.uses_held || !matches!(item.body, Body::Struct(Shape::Unit)) {
        "value"
    } else {
        "_value"
    };
    format!(
        "impl {params} {REFLECT} for {name} {arguments} {wheres} {{
            fn value_of({held}: &Self) -> {VALUE} {{ {value_of} }}
            fn apply({held}: &mut Self, {value}: &{VALUE}) -> bool {{ {apply} }}
            fn kind() -> {KIND} {{ {kind} }}
            fn fields() -> ::std::vec::Vec<{FIELD}> {{ {fields} }}
            fn about() -> &'static str {{ {about} }}
        }}",
        params = item.params,
        name = item.name,
        arguments = item.arguments,
        value_of = methods.value_of,
        apply = methods.apply,
        kind = methods.kind,
        fields = methods.fields,
        about = item.about.as_deref().unwrap_or("\"\""),
    )
}
