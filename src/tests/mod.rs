use super::*;

class! {
    pub class Gurt<T> where T: std::fmt::Display {
        let x: T;

        pub Gurt(x: T) {
            self::_default_constructor(x)
        }
        pub static fn test_gurt<K>(y: K) where K: std::fmt::Display {
            println!("YO GURT: {}", y);
        }
        pub const fn display_self<K>(y: K) where K: std::fmt::Display {
            println!("gurt... {} {}", self.x, y);
        }
    }
}

#[test]
fn it_works() {
    let gurt = Gurt::init(342);
    gurt.display_self(343u32);
}
