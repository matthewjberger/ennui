use crate::data::{Format, Parsed};
use ennui::prelude::Entity;
use ennui::reflect::prelude::Value;
use std::collections::BTreeMap;

#[derive(Clone, Default, PartialEq, Debug, ennui::Reflect)]
#[reflect(about = "One binding: a field of this entity filled from a source template")]
pub struct Entry {
    #[reflect(
        about = "The field written, as Component.field, for example Text.words, Style.opacity or Hidden"
    )]
    pub target: String,
    #[reflect(
        about = "Words and {source} parts: {Race.lap} a resource field, {subject.Health.current} a component field on the subject, {item.name} a field of the list item, {text.hud.lap} a text table entry"
    )]
    pub source: String,
    #[reflect(
        about = "Plain keeps the value, Whole rounds, Decimals[2] fixes the places, Percent shows 0.5 as 50%, Time shows seconds as mm:ss.000"
    )]
    pub format: Format,
    #[reflect(about = "Writes the target back to the source when the user changes it")]
    pub back: bool,
}

#[derive(Clone, Default, PartialEq, Debug, ennui::Reflect)]
#[reflect(
    about = "Fills fields of this entity from resources, the subject, the list item and text tables"
)]
pub struct Bind {
    #[reflect(about = "The bindings")]
    pub entries: Vec<Entry>,
}

#[derive(Clone, Copy, Default, PartialEq, Debug, ennui::Reflect)]
#[reflect(
    about = "The entity that {subject...} sources read for this entity and everything below it; the nearest Subject up the tree wins"
)]
pub struct Subject(#[reflect(about = "The subject entity")] pub Entity);

#[derive(Clone, Default, PartialEq, Debug, ennui::Reflect)]
#[reflect(
    about = "The list item that {item...} sources read for this entity and everything below it, as a record of fields"
)]
pub struct ListItem(#[reflect(about = "The item's fields")] pub Value);

#[derive(Clone, Default, PartialEq, Debug, ennui::Reflect)]
#[reflect(
    about = "Sample values for sources below here, read in the editor and when a source is unbound; nearer samples override"
)]
pub struct Samples(
    #[reflect(
        about = "Source path to sample value, for example [[\"Race.lap\" 3] [\"subject.Health.current\" 50]]"
    )]
    pub BTreeMap<String, Value>,
);

#[derive(Clone, Default)]
pub(crate) struct Prepared {
    pub bind: Bind,
    pub entries: Vec<Option<Parsed>>,
    pub written: Vec<Option<Value>>,
    pub shown: Vec<Option<Value>>,
}
