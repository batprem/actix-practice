use services_todos::add;
use core::greet;


fn main() {
    println!("Hello, world! {}", add(1, 2));
    println!("{}", greet("from cli1.rs"));
}