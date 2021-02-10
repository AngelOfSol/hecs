//! A registry for cloning [`World`](crate::World)s; requires the `clone` feature
//!
//! [`Component`](crate::Component)s are not necessarily cloneable, so we cannot directly
//! implement [`std::clone::Clone`] for [`World`](crate::World) without providing a registry
//! that maps types to clone functions. The registry defined in this module allow the
//! creation of clone-able registry which will be provided to a World upon creation,
//! enabling calls to world.clone() to work as you would expect.  Without a registry
//! initialized on World creation, world.clone() WILL panic.

use crate::alloc::vec::Vec;
use crate::{Archetype, ColumnBatchBuilder, ColumnBatchType, Component};
use core::any::type_name;
use core::any::TypeId;

/// An opaque registry that holds data that helps a World clone itself.
#[derive(Clone, Default)]
pub struct CloneRegistry(pub(crate) Vec<CloneEntry>);

impl CloneRegistry {
    /// Registers `T` with the registry, enabling `T` to be cloned in any
    /// archetypes that contain it.
    pub fn register<T: Clone + Component>(mut self) -> Self {
        if !self.0.iter().any(|item| item.type_id == TypeId::of::<T>()) {
            self.0.push(register::<T>());
        }
        self
    }
}

#[derive(Clone)]
pub(crate) struct CloneEntry {
    pub(crate) type_id: TypeId,
    pub(crate) add_type: fn(&mut ColumnBatchType) -> (),
    pub(crate) add_values: fn(&mut ColumnBatchBuilder, &Archetype) -> (),
}
fn register<T: Component + Clone>() -> CloneEntry {
    CloneEntry {
        type_id: TypeId::of::<T>(),
        add_type: |batch_type| {
            batch_type.add::<T>();
        },
        add_values: |batch, arch| {
            let mut writer = match batch.writer::<T>() {
                Some(x) => x,
                None => panic!("missing from clone {}", type_name::<T>()),
            };

            for item in match arch.get::<T>() {
                Some(x) => x,
                None => panic!("missing from archetype {}", type_name::<T>()),
            }
            .iter()
            {
                if writer.push(item.clone()).is_err() {
                    panic!()
                }
            }
        },
    }
}
