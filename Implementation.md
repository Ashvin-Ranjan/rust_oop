# Implementation Details

Here are the implementation details for how exactly the class information is converted into Rust, for those curious.

## What _is_ a Class?

A class is actually a module, with a struct, and implementation for that struct, a trait, and implementation of that trait (along
with parent traits). The items in class are separated based on whether they are static or local (defined with `static` or not) and
if they are a method or a field. Local fields are placed in the struct, and everything else is placed in the implementation. The
traits simply reflect back to the methods from the implementation for the struct.

The module does have to import data from all other classes in order to be able to reference them.

Here is an example of a compiled class `A` which inherits from another class `B`, which has generics (with mild formatting):

```rs
#[allow(non_snake_case)]
pub mod A {
    use super::B::*;
    pub struct A {
        pub val: (u32, u32),
        #[doc(hidden)]
        __phantom_marker0: ::std::marker::PhantomData<()>,
    }
    impl C {
        pub fn touch_val(&self) {
            type T = u32;
            let _x = &self.val;
        }
        pub fn init(val: u32) -> A {
            Self::_default_constructor((val, val))
        }
        fn _default_constructor(val: (u32, u32)) -> A {
            A {
                val,
                __phantom_marker0: ::std::marker::PhantomData,
            }
        }
    }
    pub trait AInstance: BInstance<u32> {
        fn touch_val(&self);
    }
    impl AInstance for A {
        fn touch_val(&self) {
            self.touch_val()
        }
    }
    impl BInstance<u32> for A {
        fn touch_val(&self) {
            self.touch_val()
        }
    }
}
```

## The Constructor

A major contrivance with the way classes work is the existence of `Self::_default_constructor`. The reason for this is actually due
to the interaction between generics and structs. Since the struct has to include the generics of the class, there are cases where
the class will need to contain a `std::marker:PhantomData` to avoid getting an error about not using the generic. This phantom data
field is actually created for all classes (and created in such a way to avoid naming conflicts), this is why it is recommended to use
`Self::_default_constructor` rather than just creating the struct outright, since the phantom data field needs to be set and while
its name is deterministic, it can change.

Here is an example of a `_default_constructor` (with mild formatting):

```rs
fn _default_constructor(val : (u32, u32),) -> A {
    A { val, __phantom_marker0 : :: std :: marker :: PhantomData, }
}
```

Outside of the existence of `Self::_default_constructor`, as discussed in the base documentation the constructor syntax is just syntactic sugar for:

```rs
static fn init(...) -> ClassName { ... }
```

## How Does Inheritance Work?

Inheritance works simply by copying the inherited methods and fields wholesale (not exactly, but this is explained more in the generics section) into
those of the current class. This is why cross-macro inheritance is not supported, as the code inside function blocks is copied from the parent. This
allows inherited functions to access fields. The class' associated instance trait is also updated to inherit from its parent's, and all relevant
implementations are made for parent traits. Each function in the parent trait acts as a wrapper around a call to the function in the struct's implementation.

## Generics

Generics are handled by constructing a map during compilation of what each generic of each parent should map to for the child. This is specifically used when
implementing the instance traits for each of the parents, as the generics need to be passed in when declaring the implementation, and the mapping needs to be
used to edit the function signatures. A mapping to each child from its direct parent is also used to modify each of the fields and methods inherited to ensure
they have the correct type. Finally, for each inherited function, the correct `type` declarations are inserted at the top of the function body in order to ensure
proper typing.

## Misc.

There are a few additional things which the macro does, namely it scopes everything inside a module called `class_container` and also creates `use` statements at
the end to allow for code outside of the macro but still in the file to interact with the class code while still maintaining encapsulation.
