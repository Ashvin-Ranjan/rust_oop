use super::*;

class! {
    pub class Root<K, T> {}

    pub class Yo<T> : Root<i32, T> {
        let pub x: i32;
        pub static fn test_yo() {
            println!("YO");
        }
        pub const fn display_yo() {
            println!("yo...");
        }
    }

    pub class Gurt<T>: Yo<T> {
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
    let gurt = Gurt::<i32>::init();
    helper(&gurt);
}

fn helper<T>(value: &dyn YoInstance<T>) {
    value.display_yo();
}
