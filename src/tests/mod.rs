use std::marker::PhantomData;

use super::*;

class! {
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

    pub class Yo {
        pub static fn test_yo() {
            println!("YO");
        }
        pub const fn display_yo() {
            Self::test_yo();
            println!("yo...");
        }
    }

    pub class Gurt : Yo {
        pub Gurt() {
            Self::_default_constructor()
        }
        pub static override fn test_yo() {
            println!("gurt: yo")
        }
        pub static fn test_gurt() {
            println!("YO GURT");
        }
        pub const fn display_self() {
            println!("gurt...");
        }
    }
}

#[test]
fn it_works() {
    let thing = C::init(3);
}

fn helper(thing: &dyn YoInstance) {
    thing.display_yo();
}

#[test]
fn generic_chain_compiles() {
    // Verify at compile time that C and D satisfy the transitively resolved trait bounds.
    // C : B<u32> : A<(u32, u32)>, so C must implement AInstance<(u32, u32)> and BInstance<u32>.
    fn assert_a<T: AInstance<(u32, u32)>>() {}
    fn assert_b<T: BInstance<u32>>() {}

    assert_a::<C>();
    assert_b::<C>();
    // D inherits from C, so it must also satisfy the same bounds.
    assert_a::<D>();
    assert_b::<D>();
}
