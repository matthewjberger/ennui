use ennui_ecs::entity::Entity;

#[derive(Clone, Debug, PartialEq, Default)]
pub enum Value {
    #[default]
    Unit,
    Bool(bool),
    Number(f64),
    Text(String),
    Word(String),
    Reference(String),
    Entity(Entity),
    List(Vec<Value>),
    Record(Vec<(String, Value)>),
    Variant(String, Box<Value>),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Save {
    #[default]
    Scene,
    Snapshot,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum Kind {
    #[default]
    Other,
    Bool,
    Number,
    Whole,
    Text,
    Vector(usize),
    Rotation,
    Color,
    Choice(Vec<&'static str>),
    Path(Vec<&'static str>),
    Entity,
    List,
    Record,
    Optional(Box<Kind>),
}

#[derive(Clone, Debug)]
pub struct Field {
    pub name: &'static str,
    pub kind: Kind,
    pub save: Save,
    pub range: Option<(f64, f64)>,
    pub step: Option<f64>,
    pub about: &'static str,
    pub fields: fn() -> Vec<Field>,
    pub item: fn() -> Option<Value>,
}

pub trait Reflect: 'static {
    fn value_of(held: &Self) -> Value;
    fn apply(held: &mut Self, value: &Value) -> bool;
    fn kind() -> Kind {
        Kind::Other
    }
    fn fields() -> Vec<Field> {
        Vec::new()
    }
    fn item() -> Option<Value> {
        None
    }
    fn about() -> &'static str {
        ""
    }
}

pub(crate) type ReadComponent = fn(&ennui_ecs::storage::Storage, Entity) -> Option<Value>;
pub(crate) type WriteComponent = fn(&mut ennui_ecs::storage::Storage, Entity, &Value) -> bool;
pub(crate) type DropComponent = fn(&mut ennui_ecs::storage::Storage, Entity);
pub(crate) type HasComponent = fn(&ennui_ecs::storage::Storage, Entity) -> bool;
pub(crate) type TouchedComponent = fn(&ennui_ecs::storage::Storage, u64) -> bool;
pub(crate) type FitsValue = fn(&Value) -> bool;
pub(crate) type MadeValue = fn() -> Value;

pub struct Settled;

#[derive(Clone)]
pub struct Described {
    pub name: &'static str,
    pub full: &'static str,
    pub key: std::any::TypeId,
    pub kind: Kind,
    pub fields: Vec<Field>,
    pub about: &'static str,
    pub read: ReadComponent,
    pub write: Option<WriteComponent>,
    pub fits: FitsValue,
    pub made: Option<MadeValue>,
    pub remove: DropComponent,
    pub has: HasComponent,
    pub touched: TouchedComponent,
}

#[derive(Clone)]
pub struct DescribedResource {
    pub name: &'static str,
    pub key: std::any::TypeId,
    pub fields: Vec<Field>,
    pub about: &'static str,
    pub fits: FitsValue,
    pub made: MadeValue,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Number(f64),
    Word(String, bool),
    Text(String),
    Reference(String),
    Open(char),
    Close(char),
}
