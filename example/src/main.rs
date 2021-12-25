::std::thread_local! {
    static DEPTH: ::std::cell::Cell<usize> = ::std::cell::Cell::new(0);
}

#[flamelines::counter]
fn dummy() -> i32 {
    println!("1");
    println!("2");
    println!("3");
    println!("4");
    dummy2();
    println!("5");
    42
}

#[flamelines::counter]
fn dummy2() -> i32 {
    println!("I'm the son");
    42
}

fn main() {
    dummy();
    println!("main finished");
}
