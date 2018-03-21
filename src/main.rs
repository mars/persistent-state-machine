extern crate chrono;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;

pub mod lifecycle;
pub mod schema;
pub mod models;

fn main() {
    println!("Hello, world!");
}
