::std::thread_local! {
    static DEPTH: ::std::cell::Cell<usize> = ::std::cell::Cell::new(0);
}

extern crate env_logger;

/// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
/// one function example
/// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
#[flamelines::time_lines]
fn normal_func() -> i32 {
    println!("1");
    println!("2");
    println!("3");
    let mut a = 4;
    a += 5;
    println!("4 {}", a);
    one_module::child_func();
    println!("5");
    42
}

/// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
/// one module example, it recursivly hooks all functions of this module
/// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[flamelines::time_lines]
mod one_module {
    pub fn child_func() -> i32 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        println!("I'm the child func");
        42
    }
}

/// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
/// async example : works great with actix !!!
/// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[flamelines::time_lines]
async fn async_parent_func() -> i32 {
    println!("I'm an async func");
    42
}

struct ImplDemo {
    name: String,
}

/// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
/// example on a impl, it hooks all methods
/// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[flamelines::time_lines]
impl ImplDemo {
    fn new() -> Self {
        println!("We are in ImplDemo::new");
        std::thread::sleep(std::time::Duration::from_secs(1));
        Self {
            name: "Robert".to_string(),
        }
    }
    async fn change_name(&mut self) {
        println!("I'n an async method of a string");
        self.name = "Jean".to_string();
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    std::env::set_var("RUST_LOG", "info");

    normal_func();
    async_parent_func().await;

    let mut impl_demo = ImplDemo::new();
    impl_demo.change_name().await;
}
