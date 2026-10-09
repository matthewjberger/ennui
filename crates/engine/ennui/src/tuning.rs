#[macro_export]
macro_rules! tuning {
    (
        $(#[$note:meta])*
        $visible:vis struct $name:ident {
            $($(#[$field_note:meta])* $field:ident: $kind:ty = $start:expr),* $(,)?
        }
    ) => {
        $(#[$note])*
        $visible struct $name {
            $($(#[$field_note])* pub $field: $kind,)*
        }

        impl Default for $name {
            fn default() -> Self {
                Self { $($field: $start,)* }
            }
        }
    };
}

#[macro_export]
macro_rules! setters {
    ($name:ident<$life:lifetime> { $($field:ident: $kind:ty),* $(,)? }) => {
        impl<$life> $name<$life> {
            $(
                pub fn $field(mut self, held: $kind) -> Self {
                    self.$field = held;
                    self
                }
            )*
        }
    };
    ($name:ident { $($field:ident: $kind:ty),* $(,)? }) => {
        impl $name {
            $(
                pub fn $field(mut self, held: $kind) -> Self {
                    self.$field = held;
                    self
                }
            )*
        }
    };
}
