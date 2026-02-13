use std::fmt::Display;

use super::*;

class! {
    class Gurt<T> where T: Display {
        let x: T;
        let static y: i32 = 67;
        pub Gurt(_x: T) {
            Gurt::_default_constructor(_x)
        }
        pub static fn test_gurt() {
            println!("YO GURT");
        }
        pub fn test_gurt_instance(x: T) {
            println!("gurt... {} {} {}", self.x, x, Gurt::<T>::y);
        }
    }
}

#[test]
fn it_works() {
    Gurt::<u32>::test_gurt();
    (Gurt::<u32>::init(342)).test_gurt_instance(12);
}
