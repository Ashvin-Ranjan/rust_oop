use super::*;

class! {
    class Gurt {
        pub static fn test_gurt() {
            println!("YO GURT");
        }
    }
}

#[test]
fn it_works() {
    Gurt::test_gurt();
}
