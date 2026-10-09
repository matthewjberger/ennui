use ennui::reflect::prelude::{Kind, Reflect, Value};

pub fn parsed_choice<Chosen: Reflect + Default>(text: &str) -> Result<Chosen, String> {
    let Kind::Choice(names) = Chosen::kind() else {
        return Err(String::from("not a choice"));
    };
    let mut held = Chosen::default();
    match names.iter().find(|name| name.eq_ignore_ascii_case(text)) {
        Some(name) if Chosen::apply(&mut held, &Value::Word(name.to_string())) => Ok(held),
        _ => Err(format!(
            "expected one of {}",
            names.join(", ").to_lowercase()
        )),
    }
}
