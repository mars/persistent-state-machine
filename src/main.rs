extern crate chrono;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

pub mod lifecycle;
pub mod schema;
pub mod models;

fn main() {
    println!("Hello, world!");
}
