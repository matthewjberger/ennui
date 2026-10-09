use crate::data::{Described, DescribedResource, Value};
use std::any::TypeId;
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Default)]
pub struct Reflected {
    pub components: Vec<Described>,
    pub resources: Vec<DescribedResource>,
    pub named: HashMap<&'static str, usize>,
    pub keyed: HashMap<TypeId, usize>,
    pub resource_named: HashMap<&'static str, usize>,
}

#[derive(Default)]
pub struct Settings {
    pub wanted: Vec<(String, Value)>,
    pub shown: BTreeMap<String, Value>,
    pub problems: Vec<String>,
    pub turn: u64,
    pub changed: BTreeMap<String, u64>,
    pub made: BTreeMap<String, Value>,
    pub reset: BTreeSet<String>,
}
