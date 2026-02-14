use super::*;

class! {
    pub class Gurt {
        let x: i32;
        let y: i32;

        pub Gurt(x: i32, y: i32) {
            self::_default_constructor(x, y)
        }
        pub static fn test_gurt() {
            println!("YO GURT");
        }
        pub const fn display_self() {
            println!("gurt... {} {}", self.x, self.y);
        }
    }
}

#[test]
fn it_works() {
    let gurt = Gurt::init(342, 20);
    gurt.display_self();
}
