#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

use std::sync::Mutex;
use std::collections::VecDeque;

use rocket::State;
use rocket_contrib::json::{Json, JsonValue};

// The type to represent the ID of a message.
type ID = usize;

// We're going to store all of the messages here. No need for a DB.
type Calculator = Mutex<VecDeque<String>>;

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    contents: VecDeque<String>
}

#[put("/push", format = "json", data = "<message>")]
fn append(message: Json<Message>, calculator: State<Calculator>) -> JsonValue {
    let mut mut_calculator = calculator.lock().unwrap();
    if message.contents.len() < 3 {
        json!({
            "status": "error",
            "reason": "size of the contents must be greater than or equal to 3"
        })
    } else {
        for content in &message.contents {
            mut_calculator.push_back(content.to_string());
        }
        json!({ "status": "ok" })
    }
}

#[get("/calculate", format = "json")]
fn calculate(calculator: State<Calculator>) -> JsonValue {
    let mut mut_calculator = calculator.lock().unwrap();
    let mut mut_temp_stack = VecDeque::<String>::new();
    while mut_calculator.len() > 0 {
        let front_pop = mut_calculator.pop_front();
        match front_pop {
            Some(popped) => {
                if is_int(&popped) {
                    mut_temp_stack.push_back(popped)
                } else if is_operator(&popped) {
                    if mut_temp_stack.len() > 1 {
                        let second = mut_temp_stack.pop_back();
                        let first = mut_temp_stack.pop_back();
                        let result = action(
                            first.unwrap().parse::<i64>().unwrap(),
                            second.unwrap().parse::<i64>().unwrap(),
                            &popped
                        );
                        mut_temp_stack.push_back(result.to_string());
                    } else {
                        println!("Temporary stack is size 1 or less.");
                    }
                } else {
                    println!("Not an int or an operator.");
                }
            }
            None => println!("Nothing popped.")
        }
    }
    let result = match mut_temp_stack.pop_front() {
        Some(val) => val.to_string(),
        None => "Nothing to calculator on the stack".to_string(),
    };
    json!({ "status":  result})
}

fn action(first: i64, second: i64, operator: &String) -> i64 {
    return match String::as_ref(&operator) {
        "*" => first * second,
        "/" => first / second,
        "+" => first + second,
        "-" => first - second,
        // shouldn't hit this case as we verify with is_operator
        _ => 0,
    };
}

fn is_int(popped: &String) -> bool {
    return match popped.parse::<i64>() {
        Ok(parsed) => true,
        Err(e) => false,
    };
}

fn is_operator(popped: &String) -> bool {
    return match String::as_ref(&popped) {
        "*" | "/" | "+" | "-" => true,
        _ => false,
    };
}

#[get("/", format = "json")]
fn trace(calculator: State<Calculator>) -> Json<Message> {
    let mut mut_calculator = calculator.lock().unwrap();
    let mut mut_vec = VecDeque::<String>::new();
    for content in mut_calculator.iter() {
        mut_vec.push_back(content.to_string());
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
        .manage(Mutex::new(VecDeque::<String>::new()))
}

fn main() {
    rocket().launch();
}
