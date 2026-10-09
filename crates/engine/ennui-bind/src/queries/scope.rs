use crate::components::{ListItem, Samples, Subject};
use crate::data::Scope;
use ennui::prelude::{Entity, Storage};
use ennui::reflect::prelude::Value;
use ennui::storage::get;
use ennui_scene::prelude::ancestors;

pub fn scope_of(storage: &Storage, entity: Entity) -> Scope {
    let mut scope = Scope {
        subject: None,
        item: None,
        samples: Vec::new(),
    };
    for at in ancestors(storage, entity) {
        if scope.subject.is_none()
            && let Some(subject) = get::<Subject>(storage, at)
        {
            scope.subject = Some(subject.0);
        }
        if scope.item.is_none() && get::<ListItem>(storage, at).is_some() {
            scope.item = Some(at);
        }
        if get::<Samples>(storage, at).is_some() {
            scope.samples.push(at);
        }
    }
    scope
}

pub fn sample_of(storage: &Storage, scope: &Scope, raw: &str) -> Option<Value> {
    scope
        .samples
        .iter()
        .find_map(|holder| get::<Samples>(storage, *holder)?.0.get(raw).cloned())
}
