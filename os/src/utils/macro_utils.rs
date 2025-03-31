//! marco utils:
//! quick macro for generating code

#[macro_export]
/// useful for struct like: struct XX { inner: SomeLock<XXInner> }
macro_rules! with_methods {
    ($($name:ident : $ty:ty),+) => {
        paste::paste! {
            $(
                pub fn [<with_ $name>]<T>(&self, f: impl FnOnce(&$ty) -> T) -> T {
                    f(&self.$name.lock())
                }
                pub fn [<with_mut_ $name>]<T>(&self, f: impl FnOnce(&mut $ty) -> T) -> T {
                    f(&mut self.$name.lock())
                }
            )+
        }
    };
}