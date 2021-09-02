use std::collections::HashMap;

use rocket::serde::json::serde_json::json;
use rocket::serde::json::{Json, Value};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::sync::Mutex;
use rocket::{catch, catchers, delete, get, post, put, routes, State};

#[get("/")]
async fn hello() -> String {
    "hello world".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
struct Task {
    id: usize,
    name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(crate = "rocket::serde")]
struct Error {
    code: usize,
    reason: String,
    description: String,
}

// restful
#[get("/ex")]
async fn get_exs() -> Value {
    json!({"data": "ex list".to_string()})
}

#[get("/ex/<id>")]
async fn get_ex(id: usize) -> Value {
    json!({"data": id.to_string()})
}

#[post("/ex", format = "json", data = "<task>")]
async fn post_ex(task: Json<Task>) -> Value {
    json!({"data": task.into_inner()})
}

#[put("/ex/<id>")]
async fn put_ex(id: usize) -> Value {
    json!({"data": id.to_string()})
}

#[delete("/ex/<id>")]
async fn delete_ex(id: usize) -> String {
    id.to_string()
}

#[catch(404)]
async fn not_found() -> Value {
    json!({"error": Error{code: 404, ..Default::default() }})
}

#[catch(404)]
async fn not_found_base() -> Value {
    json!({"error": Error{code: 404, description: "not found base".to_string(), ..Default::default()}})
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
struct Person {
    id: usize,
    name: String,
    age: u8,
}

type PersonItems = Mutex<HashMap<usize, Person>>;
type Messages<'r> = &'r State<PersonItems>;

#[get("/people/<id>")]
async fn get_person(id: usize, messages: Messages<'_>) -> Json<Person> {
    let person_data = messages.lock().await;
    if id == 0 {
        return Json(Person {
            id: 0,
            name: "_".to_string(),
            age: 0,
        });
    }
    match person_data.get(&id) {
        None => Json(Person {
            id: 0,
            name: "".to_string(),
            age: 0,
        }),
        Some(key) => Json(key.to_owned()),
    }
}

#[post("/people", format = "json", data = "<person>")]
async fn post_person(person: Json<Person>, messages: Messages<'_>) -> Value {
    let mut person_data = messages.lock().await;
    let raw_person = person.into_inner();
    if person_data.contains_key(&raw_person.id) {
        json!({"error": Error{code: 400, description: "person already exists".to_string(), ..Default::default() }})
    } else {
        person_data.insert(raw_person.id, raw_person);
        json!({ "data": "ok" })
    }
}

#[put("/people/<id>", format = "json", data = "<person>")]
async fn put_person(id: usize, person: Json<Person>, messages: Messages<'_>) -> Value {
    let mut person_data = messages.lock().await;
    let mut raw_person = person.into_inner();
    if person_data.contains_key(&id) {
        raw_person.id = id;
        person_data.insert(id, raw_person);
        json!({ "data": "ok" })
    } else {
        json!({"error": Error{code: 400, description: "person hasnot been found".to_string(), ..Default::default() }})
    }
}

#[delete("/people/<id>")]
async fn delete_person(id: usize, messages: Messages<'_>) -> Value {
    let mut person_data = messages.lock().await;
    if person_data.contains_key(&id) {
        let raw_person = person_data.get(&id).unwrap().to_owned();
        person_data.remove(&id);
        json!({ "data": raw_person })
    } else {
        json!({"error": Error{code: 400, description: "person hasnot been found".to_string(), ..Default::default() }})
    }
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    rocket::build()
        // route
        .mount("/hello", routes![hello])
        .mount(
            "/base",
            routes![get_exs, get_ex, post_ex, put_ex, delete_ex],
        )
        .mount(
            "/rust",
            routes![get_person, post_person, put_person, delete_person,],
        )
        // catch
        .register("/", catchers![not_found])
        .register("/base", catchers![not_found_base])
        // state
        .manage(PersonItems::new(HashMap::new()))
        .launch()
        .await
}
