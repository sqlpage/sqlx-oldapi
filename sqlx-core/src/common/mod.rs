mod statement_cache;

pub(crate) use statement_cache::StatementCache;
#[cfg(feature = "sqlite")]
use std::fmt::{Debug, Formatter};
#[cfg(feature = "sqlite")]
use std::ops::{Deref, DerefMut};

/// A wrapper for `Fn`s that provides a debug impl that just says "Function"
#[cfg(feature = "sqlite")]
pub(crate) struct DebugFn<F: ?Sized>(pub F);

#[cfg(feature = "sqlite")]
impl<F: ?Sized> Deref for DebugFn<F> {
    type Target = F;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "sqlite")]
impl<F: ?Sized> DerefMut for DebugFn<F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(feature = "sqlite")]
impl<F: ?Sized> Debug for DebugFn<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Function").finish()
    }
}
