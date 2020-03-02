#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

use std::sync::Mutex;
use std::vec::Vec;

use rocket::State;
use rocket_contrib::json::{Json, JsonValue};

// The type to represent the ID of a message.
type ID = usize;

// We're going to store all of the messages here. No need for a DB.
type Calculator = Mutex<Vec<String>>;

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    contents: Vec<String>
}

#[put("/push", format = "json", data = "<message>")]
fn append(message: Json<Message>, calculator: State<Calculator>) -> JsonValue {
    let mut mut_calculator = calculator.lock().unwrap();
    println!("{:?}", message);
    if message.contents.len() < 3 {
        json!({
            "status": "error",
            "reason": "size of the contents must be greater than or equal to 3"
        })
    } else {
        for content in &message.contents {
            mut_calculator.push(content.to_string());
        }
        println!("{}", mut_calculator.len());
        json!({ "status": "ok" })
    }
}

#[get("/calculate", format = "json")]
fn calculate(calculator: State<Calculator>) -> JsonValue {
    let mut mut_calculator = calculator.lock().unwrap();
    json!({ "status": "ok" })
}

#[get("/", format = "json")]
fn trace(calculator: State<Calculator>) -> Json<Message> {
    let mut mut_calculator = calculator.lock().unwrap();
    let mut mut_vec = Vec::<String>::new();
    println!("{}", mut_calculator.len());
    for content in mut_calculator.iter() {
        mut_vec.push(content.to_string());
    }
    Json(Message {
        contents: mut_vec
    })
}

#[catch(404)]
fn not_found() -> JsonValue {
    json!({
        "status": "error",
        "reason": "Resource was not found."
    })
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![append, calculate, trace])
        .register(catchers![not_found])
        .manage(Mutex::new(Vec::<String>::new()))
}

fn main() {
    rocket().launch();
}
