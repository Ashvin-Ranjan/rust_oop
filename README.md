# Super Legitimate (Object) Oriented Programming (`sl_op`)

The best use case for rust :)

## Using `sl_op`

### The `class!` Macro

Everything in `sl_op` is based around the `class!` macro. This macro takes in a list of class definitions, and creates
both types named after the class, and traits named after the class called `[ClassName]Instance` which can be used for
dynamic dispatch.

Here is an example:

```rs
class! {
    class Shape {}
    class Square {}
    class Triangle {}
}
```

### Class Definitions

A class definition uses the `class` keyword and a name directly afterwards (inheritance and generics will be discussed later).
Class definitions can be modified using the `pub` keyword at the beginning. The `pub` keyword allows the class and its trait
to be referenced outside of the `class!` macro scope. Inside the curly braces is the class body, which contains definitions and
declarations of both methods and fields.

### Fields

fields are defined by `let` and can either be local (which we shall cover here) or static (which will be covered later).
Local fields simply have a name and a type, and end with a semicolon. These can also be modified by the `pub` keyword to
allow code outside of the object to be able to access and modify the field. Here is an example of a class called `Point`, which
has a public x and y coordinate and a private id value:

```rs
class Point {
    let pub x: f32;
    let pub y: f32;
    let id: usize;
}
```

Now if there exists an instance of `Point` called `p`, `p.x` and `p.y` would be valid, but `p.id` would not be.

> [!TIP]
> Note that the mutability of the field from external code is based on whether the instance is defined as mutable or not, and cannot
> be controlled inside the class. If you do not want fields to be mutable, it is best to make them private and use getter functions.

### Methods

Methods can be defined similarly to regular functions, however `sl_op` provides the additional keyword of `const`. This comes after `pub`
and indicates that a function will not modify its own fields. A unique aspect of non-static methods is that the receiver, `&self` or `&mut self`,
is automatically provided in the function and does not have to be included in the method arguments. Whether the receiver is mutable or not is dependent
on whether the function is `const` or not. Here is an example of a `const` and non-`const` method for `Point`:

```rs
class Point {
    let pub x: f32;
    let pub y: f32;
    let id: usize;
    pub const fn get_id() -> usize { self.id }
    pub fn increment_x() { self.x += 1.0; }
}
```

> [!NOTE]
> One may ask why I chose to make mutability the default instead of using the more Rust-like paradigm of requiring explicit mutability.
> The answer is that I didn't think of that until I wrote this section.

### The `static` Keyword

The `static` keyword can be applied to both fields and methods in order to make them available class-wide as opposed to attached to a single
instance. For fields, the `static` keyword comes after `pub` and now requires the field to be explicitly defined (it is also recommended that
the field name is switched to all capitals). For methods, `static` also comes after `pub` and cannot be used with `const`, this is because in
`static` functions the receiver is no longer passed, so `&self` and `&mut self` are not accessible. In order to access a `static` field or
method in a class, you can use `Self::`. Here is an example of the point class from before, but with a `static` method and field:

```rs
class Point {
    let pub x: f32;
    let pub y: f32;
    let pub static Z_COORD: f32 = 1.0;
    let id: usize;
    pub const fn get_id() -> usize { self.id }
    pub fn increment_x() { self.x += 1.0; }
    pub static fn get_z() -> f32 { Self::Z_COORD }
}
```

### The Constructor

Defining classes is not useful as long as we are unable to declare them. To this end, `sl_op` equips every class with a private, static function
called `_default_constructor` which creates an instance of the class when passed in values for each of the fields. The order of arguments in the
constructor is the order of the fields in the class, and then the inherited fields in the order of their parent constructor.

> [!CAUTION]
> You can theoretically directly instantiate a struct instance of the class. This is not recommended for various reasons.
> For those curious, look in the [Implementation Notes](./Implementation.md) for more technical details on this.

Since defining a constructor method is common, `sl_op` provides special constructor syntax, here is a constructor for the class `Point`:

```rs
pub Point(x: f32, y: f32, id: usize) {
    Self::_default_constructor(x, y, id)
}
```

Note that under-the-hood, the constructor syntax is directly equivalent to:

```rs
static fn init(...) -> Point { ... }
```

### Inheritance

One of the more exciting parts of OOP is the use of inheritance. `sl_op` makes it simple. In the class declaration, adding `: [Parent Class]`
will inherit from the parent class (note that you can only inherit from one class). This will automatically update the fields and methods in
the class and properly inherit them based on the behavior of the parent (this includes methods that mutate fields). Here is an example of a
`Shape` and `Square` class:

```rs
pub class Shape {
    let sides: i32;
    pub const fn get_sides() -> i32 { self.sides }
    pub Shape(sides: i32) {
        Self::_default_constructor(sides)
    }
}
pub class Square : Shape {
    pub Square() {
        Self::_default_constructor(4)
    }
}
```

Note that constructors are not inherited from parent classes.

Using the `override` keyword you can override inherited methods and `static` fields. There are considerations about error checking regarding
`override`, which can be seen near the end of the documentation. Here is an example of overriding:

```rs
pub class Shape {
    let sides: i32;
    pub const fn get_sides() -> i32 { self.sides }
}
pub class Square : Shape {
    pub const override fn get_sides() -> i32 { 4 }
}
```

### Dynamic Dispatch

Each class creates a trait called `[Class Name]Instance`. This means that you can use generic types which implement `[Class Name]Instance`
or `&dyn [Class Name]Instances`. Note that these are limited to just method calls, you cannot access fields without getters and setters.

Here is an example of dynamic dispatch with `Square` and `Shape` (different from the ones above but close enough):

```rs
fn print_sides(shape: &dyn ShapeInstance) {
    println!("{}", shape.get_sides())
}

fn main() {
    let square = Square::init();
    let triangle = Shape::init(3);
    print_sides(&triangle); // 3
    print_sides(&square); // 4
}
```

### Generics

> [!CAUTION]
> Generics are VERY fragile and there are probably still many issues with them. You should probably not use them!

Classes can define any number of generics after their class name, and can also include a `where` clause after declaring
inheritances. When inheriting from a class with generics, the generics will need to be passed in. Here is an example
showing inheritance with generics:

```rs
pub class A<K> {
    let pub val: K;
    pub const fn touch_val() {
        let _x = &self.val;
    }
}

pub class B<T> : A<(T, T)> {}

pub class C : B<u32> {
    pub C(val: u32) {
        Self::_default_constructor((val, val))
    }
}

pub class D : C {}
```

### Further Information

To learn more about `sl_op` check out the [Implementation Details](./Implementation.md).

## Known Issues/Potential Problems

> [!CAUTION]
> This is **NOT** a comprehensive list of reported issues. These are only issues which I have thought of/encountered before.
> If you try to use this project seriously (which you probably shouldn't, but let me know if you do), you will probably encounter
> a lot more issues!

### Importing

Currently the classes will not be able to use any of the packages you have imported from above the macro, and they do not support
importing inside the macro. This means that you will have to qualify all imported things you use with their full paths every time.
I will attempt to fix this at some point in the future but it may take a bit.

### Mutli-File Structuring

Theoretically having `class!` blocks across multiple files should work, but this has not been tested at all. Furthermore, due to
scoping limitations, cross-macro inheritance is impossible because separate macro calls cannot share the needed information for
inheritance.

### Generics

Generics were one of the hardest parts of this implementation and I honestly almost gave up multiple times on implementing them.
In the end I did very regretfully slop some code up for this based on a theoretical implementation I had half-implemented previously.
While I did look over the code and confirm its validity, there are still most likely some major holes in the implementation of generics,
so use them with a lot of caution.

### Dynamic Dispatch Limitations

Dynamic dispatch is currently enabled through `&dyn ClassNameInstance`. This does mean however that direct field access for dynamically-dispatched
objects is impossible, so getters and setters are needed for this. I have been playing around with a custom vtable implementation which would completely
supersede Rust's dynamic dispatch routine to get around this issue, however, it is still in the works (specifically generics hell).

### Debugging

The Rust compiler does a good job of identifying which tokens are the source of an error within the `class!` macro. However, there are many errors which
will be extremely hard to debug without a good understanding of the internal structure of the `class!` macro compiler, which can be found in the
[Implementation Details](./Implementation.md).

### Error Checking With Overrides

When overriding a function, the type signatures of each function are not checked for a match. This does mean that you can override a function but take in
different arguments and the `class!` macro compiler will not throw an error. Most likely an error will be thrown in the generate code for inherited trait
implementation, however.

### Attributes

The current parser for the `class!` macro cannot handle attributes attached to fields and functions, this may be implemented at a later time.
