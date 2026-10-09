use crate::data::{Field, Kind, Reflect, Value};
use crate::queries::number::number_of;
use ennui_ecs::entity::Entity;
use nalgebra_glm::{TVec, Vec2, Vec3, Vec4};
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Arc;

impl Reflect for bool {
    fn value_of(held: &Self) -> Value {
        Value::Bool(*held)
    }
    fn apply(held: &mut Self, value: &Value) -> bool {
        let wanted = match value {
            Value::Bool(flag) => *flag,
            Value::Word(word) if word == "true" || word == "on" => true,
            Value::Word(word) if word == "false" || word == "off" => false,
            Value::Number(number) => *number != 0.0,
            _ => return false,
        };
        *held = wanted;
        true
    }
    fn kind() -> Kind {
        Kind::Bool
    }
}

impl Reflect for f32 {
    fn value_of(held: &Self) -> Value {
        Value::Number(held.to_string().parse().unwrap_or(f64::from(*held)))
    }
    fn apply(held: &mut Self, value: &Value) -> bool {
        match number_of(value) {
            Some(number) => {
                *held = number as f32;
                true
            }
            None => false,
        }
    }
    fn kind() -> Kind {
        Kind::Number
    }
}

impl Reflect for f64 {
    fn value_of(held: &Self) -> Value {
        Value::Number(*held)
    }
    fn apply(held: &mut Self, value: &Value) -> bool {
        match number_of(value) {
            Some(number) => {
                *held = number;
                true
            }
            None => false,
        }
    }
    fn kind() -> Kind {
        Kind::Number
    }
}

macro_rules! whole {
    ($($kind:ty),*) => {
        $(
            impl Reflect for $kind {
                fn value_of(held: &Self) -> Value {
                    Value::Number(*held as f64)
                }
                fn apply(held: &mut Self, value: &Value) -> bool {
                    match number_of(value) {
                        Some(number) => {
                            *held = number.round() as $kind;
                            true
                        }
                        None => false,
                    }
                }
                fn kind() -> Kind {
                    Kind::Whole
                }
            }
        )*
    };
}

whole!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

impl Reflect for String {
    fn value_of(held: &Self) -> Value {
        Value::Text(held.clone())
    }
    fn apply(held: &mut Self, value: &Value) -> bool {
        match value {
            Value::Text(text) | Value::Word(text) => {
                held.clone_from(text);
                true
            }
            Value::Number(number) => {
                *held = number.to_string();
                true
            }
            _ => false,
        }
    }
    fn kind() -> Kind {
        Kind::Text
    }
}

impl Reflect for () {
    fn value_of(_held: &Self) -> Value {
        Value::Unit
    }
    fn apply(_held: &mut Self, _value: &Value) -> bool {
        true
    }
}

impl Reflect for Value {
    fn value_of(held: &Self) -> Value {
        held.clone()
    }
    fn apply(held: &mut Self, value: &Value) -> bool {
        held.clone_from(value);
        true
    }
}

impl<T: 'static> Reflect for PhantomData<T> {
    fn value_of(_held: &Self) -> Value {
        Value::Unit
    }
    fn apply(_held: &mut Self, _value: &Value) -> bool {
        true
    }
}

impl Reflect for Entity {
    fn value_of(held: &Self) -> Value {
        Value::Entity(*held)
    }
    fn apply(held: &mut Self, value: &Value) -> bool {
        match value {
            Value::Entity(entity) => {
                *held = *entity;
                true
            }
            _ => false,
        }
    }
    fn kind() -> Kind {
        Kind::Entity
    }
}

impl<T: Reflect + Default> Reflect for Option<T> {
    fn value_of(held: &Self) -> Value {
        match held {
            Some(inner) => T::value_of(inner),
            None => Value::Word(String::from("none")),
        }
    }
    fn apply(held: &mut Self, value: &Value) -> bool {
        if matches!(value, Value::Word(word) if word == "none") {
            *held = None;
            return true;
        }
        match held {
            Some(inner) => T::apply(inner, value),
            None => {
                let mut inner = T::default();
                let taken = T::apply(&mut inner, value);
                if taken {
                    *held = Some(inner);
                }
                taken
            }
        }
    }
    fn kind() -> Kind {
        Kind::Optional(Box::new(T::kind()))
    }
    fn fields() -> Vec<Field> {
        T::fields()
    }
    fn item() -> Option<Value> {
        T::item()
    }
}

impl<T: Reflect + Default> Reflect for Vec<T> {
    fn value_of(held: &Self) -> Value {
        Value::List(held.iter().map(T::value_of).collect())
    }
    fn apply(held: &mut Self, value: &Value) -> bool {
        let Value::List(items) = value else {
            return false;
        };
        held.truncate(items.len());
        let mut taken = true;
        for (place, item) in items.iter().enumerate() {
            if place == held.len() {
                held.push(T::default());
            }
            taken &= T::apply(&mut held[place], item);
        }
        taken
    }
    fn kind() -> Kind {
        Kind::List
    }
    fn fields() -> Vec<Field> {
        T::fields()
    }
    fn item() -> Option<Value> {
        Some(T::value_of(&T::default()))
    }
}

impl<T: Reflect, const N: usize> Reflect for [T; N] {
    fn value_of(held: &Self) -> Value {
        Value::List(held.iter().map(T::value_of).collect())
    }
    fn apply(held: &mut Self, value: &Value) -> bool {
        let Value::List(items) = value else {
            return false;
        };
        let mut taken = true;
        for (slot, item) in held.iter_mut().zip(items.iter()) {
            taken &= T::apply(slot, item);
        }
        taken
    }
    fn kind() -> Kind {
        Kind::List
    }
    fn fields() -> Vec<Field> {
        T::fields()
    }
}

macro_rules! forwards {
    ($holder:ident, $opened:path $(, $bound:path)*) => {
        impl<T: Reflect $(+ $bound)*> Reflect for $holder<T> {
            fn value_of(held: &Self) -> Value {
                T::value_of(held)
            }
            fn apply(held: &mut Self, value: &Value) -> bool {
                T::apply($opened(held), value)
            }
            fn kind() -> Kind {
                T::kind()
            }
            fn fields() -> Vec<Field> {
                T::fields()
            }
            fn item() -> Option<Value> {
                T::item()
            }
        }
    };
}

forwards!(Box, Box::as_mut);
forwards!(Arc, Arc::make_mut, Clone);

macro_rules! tuples {
    ($($kind:ident $place:tt),+) => {
        impl<$($kind: Reflect),+> Reflect for ($($kind,)+) {
            fn value_of(held: &Self) -> Value {
                Value::List(vec![$($kind::value_of(&held.$place)),+])
            }
            fn apply(held: &mut Self, value: &Value) -> bool {
                let Value::List(items) = value else {
                    return false;
                };
                let mut taken = true;
                $(
                    if let Some(item) = items.get($place) {
                        taken &= $kind::apply(&mut held.$place, item);
                    }
                )+
                taken
            }
            fn kind() -> Kind {
                Kind::List
            }
        }
    };
}

tuples!(A 0, B 1);
tuples!(A 0, B 1, C 2);
tuples!(A 0, B 1, C 2, D 3);

macro_rules! pairs {
    ($map:ident, $sorted:literal $(, $bound:path)*) => {
        impl<K: Reflect + Default $(+ $bound)*, V: Reflect + Default> Reflect for $map<K, V> {
            fn value_of(held: &Self) -> Value {
                let mut pairs: Vec<(Value, Value)> = held
                    .iter()
                    .map(|(key, inner)| (K::value_of(key), V::value_of(inner)))
                    .collect();
                if $sorted {
                    pairs.sort_by(|first, second| {
                        crate::queries::text::written(&first.0)
                            .cmp(&crate::queries::text::written(&second.0))
                    });
                }
                Value::List(
                    pairs
                        .into_iter()
                        .map(|(key, inner)| Value::List(vec![key, inner]))
                        .collect(),
                )
            }
            fn apply(held: &mut Self, value: &Value) -> bool {
                let Value::List(items) = value else {
                    return false;
                };
                let mut taken = true;
                let mut made_pairs = $map::new();
                for item in items {
                    let Value::List(pair) = item else {
                        taken = false;
                        continue;
                    };
                    let (Some(key), Some(inner)) = (pair.first(), pair.get(1)) else {
                        taken = false;
                        continue;
                    };
                    let mut made_key = K::default();
                    taken &= K::apply(&mut made_key, key);
                    let mut made = V::default();
                    taken &= V::apply(&mut made, inner);
                    made_pairs.insert(made_key, made);
                }
                *held = made_pairs;
                taken
            }
            fn kind() -> Kind {
                Kind::List
            }
            fn item() -> Option<Value> {
                Some(Value::List(vec![
                    K::value_of(&K::default()),
                    V::value_of(&V::default()),
                ]))
            }
        }
    };
}

pairs!(HashMap, true, Eq, Hash);
pairs!(BTreeMap, false, Ord);

fn numbers<const N: usize>(value: &Value) -> Option<[f64; N]> {
    let Value::List(items) = value else {
        return None;
    };
    if items.len() != N {
        return None;
    }
    let mut held = [0.0; N];
    for (slot, item) in held.iter_mut().zip(items.iter()) {
        *slot = number_of(item)?;
    }
    Some(held)
}

fn listed(values: &[f32]) -> Value {
    Value::List(
        values
            .iter()
            .map(|value| Value::Number(value.to_string().parse().unwrap_or(f64::from(*value))))
            .collect(),
    )
}

macro_rules! vectors {
    ($($kind:ty, $count:literal, $shown:ident);+ $(;)?) => {
        $(
            impl Reflect for $kind {
                fn value_of(held: &Self) -> Value {
                    $shown(held.as_slice())
                }
                fn apply(held: &mut Self, value: &Value) -> bool {
                    match numbers::<$count>(value) {
                        Some(values) => {
                            *held = TVec::<f64, $count>::from(values).cast();
                            true
                        }
                        None => false,
                    }
                }
                fn kind() -> Kind {
                    Kind::Vector($count)
                }
            }
        )+
    };
}

vectors!(Vec2, 2, listed; Vec3, 3, listed);

impl Reflect for Vec4 {
    fn value_of(held: &Self) -> Value {
        listed(held.as_slice())
    }
    fn apply(held: &mut Self, value: &Value) -> bool {
        if let Some([x, y, z]) = numbers::<3>(value) {
            *held = Vec4::new(x as f32, y as f32, z as f32, 1.0);
            return true;
        }
        match numbers::<4>(value) {
            Some(values) => {
                *held = Vec4::from(values.map(|value| value as f32));
                true
            }
            None => false,
        }
    }
    fn kind() -> Kind {
        Kind::Vector(4)
    }
}
