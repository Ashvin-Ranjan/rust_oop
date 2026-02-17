use super::*;

class! {
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
    let gurt = Gurt::init();
    helper(&gurt);
}

fn helper(thing: &dyn YoInstance) {
    thing.display_yo();
}
