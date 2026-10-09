use super::Component;
use super::Storage;
use crate::entity::Entity;

pub trait Bundle {
    fn write(self, storage: &mut Storage, entity: Entity);
}

macro_rules! bundles {
    ($first:ident $held:ident $(, $kind:ident $rest:ident)*) => {
        impl<$first: Component $(, $kind: Component)*> Bundle for ($first, $($kind,)*) {
            fn write(self, storage: &mut Storage, entity: Entity) {
                let ($held, $($rest,)*) = self;
                crate::storage::set(&mut *storage, entity, $held);
                $(crate::storage::set(&mut *storage, entity, $rest);)*
            }
        }
    };
}

bundles!(A a);
bundles!(A a, B b);
bundles!(A a, B b, C c);
bundles!(A a, B b, C c, D d);
bundles!(A a, B b, C c, D d, E e);
bundles!(A a, B b, C c, D d, E e, F f);
bundles!(A a, B b, C c, D d, E e, F f, G g);
bundles!(A a, B b, C c, D d, E e, F f, G g, H h);
