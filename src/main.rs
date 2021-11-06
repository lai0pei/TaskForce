#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
use dotenv::dotenv;
use pwhash::sha1_crypt::hash;
use rocket::config::{Config, Environment, Value};
use rocket_contrib::databases::diesel::*;
use rocket_contrib::json::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

const SUCCESS: &'static str = "Success";

#[derive(Serialize, Deserialize)]
enum Message {
    SUCCESS,
    FAIL,
}

#[database("DB")]
struct Db(diesel::PgConnection);

#[allow(unused)]
#[derive(Deserialize, Insertable)]
#[table_name = "admin"]
pub struct RegisterAdmin {
    user_name: String,
    password: String,
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct AllAdmin {
    id: i32,
    user_name: String,
    password: String,
    // create_time : String,
}

table! {
  admin(id) {
    id -> Serial,
    user_name -> VarChar,
    password -> VarChar,
    // create_time -> Text,
  }
}

#[derive(Serialize, Deserialize)]
struct Msg<T: Sized> {
    msg: Message,
    data: T,
}

fn msg<T: Sized>(message: Message, response: T) -> Msg<T> {
    Msg {
        msg: message,
        data: response,
    }
}

#[allow(unused)]
#[get("/login/<name>/<pass>")]
fn login(name: String, pass: String, db: Db) -> Json<Msg<String>> {
    let username = name.as_str();
    let password = pass.as_str();
    let res = admin::table
        .select((admin::user_name, admin::password))
        .filter(
            admin::user_name
                .eq(username)
                .and(admin::user_name.eq(password)),
        )
        .first::<(String, String)>(&*db)
        .unwrap();
    if hash(password).unwrap() == res.1 && username == res.0 {
        Json(msg(Message::SUCCESS, res.1))
    } else {
        Json(msg(Message::FAIL, "credential not correct".to_owned()))
    }
}

#[allow(unused)]
#[get("/register/<user>/<pass>")]
fn register(user: String, pass: String, db: Db) -> Json<Msg<String>> {
    if !user.is_empty() {
        let status = admin::table
            .select(admin::user_name)
            .filter(admin::user_name.eq(user.to_owned()))
            .first::<(String)>(&*db);
        match status {
            Ok(val) => return Json(msg(Message::SUCCESS, "User name already taken".to_string())),
            Err(val) => {
                let hash_pass = hash(pass.to_string()).unwrap();
                let new_user: RegisterAdmin = RegisterAdmin {
                    user_name: user,
                    password: hash_pass,
                };
                let res = insert_into(admin::table)
                    .values(&new_user)
                    .execute(&*db)
                    .unwrap();
                return Json(msg(
                    Message::SUCCESS,
                    "account created successfully".to_string(),
                ));
            }
        }
    } else {
        Json(msg(
            Message::FAIL,
            "account created successfully".to_string(),
        ))
    }
}

#[get("/")]
fn show(db: Db) -> Json<Msg<Vec<AllAdmin>>> {
    let res = admin::table.load::<AllAdmin>(&*db).unwrap();
    Json(msg(Message::SUCCESS, res))
}

#[allow(unused)]
fn env_config() -> Config {
    dotenv().ok();
    let mut extras = HashMap::new();
    let mut rocket_url: HashMap<String, String> = HashMap::new();
    let db_url = env::var("DB_CONNECTION").unwrap();
    rocket_url.insert("url".to_string(), db_url.to_string());
    extras.insert("DB".to_string(), Value::from(rocket_url));
    let mut config = Config::build(Environment::Development)
        .extra("databases", extras)
        .unwrap();
    config
}

#[allow(unused)]
fn show_data<T: std::fmt::Debug>(data: T) {
    println!("Data {:?}", data);
}

fn main() {
    rocket::custom(env_config())
        .attach(Db::fairing())
        .mount("/", routes![register, login, show])
        .launch();
}
