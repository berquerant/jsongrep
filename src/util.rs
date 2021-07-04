use std::any;

/// Return the name of the type `T`.
pub(crate) fn type_name<T>(_: T) -> &'static str {
    any::type_name::<T>()
}
