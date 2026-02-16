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
            Self::_default_constructor(324)
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

fn helper(value: &dyn YoInstance) {
    value.display_yo();
}
