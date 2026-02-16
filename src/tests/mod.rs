use super::*;

class! {
    pub class Yo {
        let pub x: i32;
        pub static fn test_yo() {
            println!("YO");
        }
        pub const fn display_yo() {
            println!("yo...");
        }
    }

    pub class Gurt: Yo {
        pub Gurt() {
            self::_default_constructor(34)
        }
        pub static fn test_gurt() {
            test_yo();
            println!("YO GURT");
        }
        pub const fn display_self() {
            self.display_yo();
            println!("gurt...");
        }
    }
}

#[test]
fn it_works() {
    let gurt = Gurt::init();
    gurt.display_self();
    Gurt::test_gurt();
}
