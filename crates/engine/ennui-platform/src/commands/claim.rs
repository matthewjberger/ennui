use std::any::TypeId;
use std::collections::HashSet;

pub fn set_claim<Source: 'static>(claims: &mut HashSet<TypeId>, held: bool) {
    let source = TypeId::of::<Source>();
    match held {
        true => claims.insert(source),
        false => claims.remove(&source),
    };
}
