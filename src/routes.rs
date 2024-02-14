use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::form::Form;
use rocket::serde::Serialize;
use rocket_dyn_templates::Template;

use crate::DbConn;
use crate::db_structs::users::*;
use crate::db_structs::pings::*;
use crate::helpers;

#[post("/", data = "<user_form>")]
pub async fn new(user_form: Form<NewUser>, conn: DbConn) -> Flash<Redirect> {
    let newuser = user_form.into_inner();
    if newuser.name.is_empty() {
        Flash::error(Redirect::to("/admin"), "Name cannot be empty.")
    } else if let Err(e) = User::insert(newuser, helpers::new_link(8), &conn).await {
        error_!("DB insertion error: {}", e);
        Flash::error(Redirect::to("/admin"), "User could not be inserted due an internal error.")
    } else {
        Flash::success(Redirect::to("/admin"), "User successfully added.")
    }
}

#[put("/<id>")]
pub async fn toggle(id: i32, conn: DbConn) -> Result<Redirect, Template> {
    match User::toggle_with_id(id, &conn).await {
        Ok(_) => Ok(Redirect::to("/admin")),
        Err(e) => {
            error_!("DB toggle({}) error: {}", id, e);
            Err(Template::render("index", Context::err(&conn, "Failed to toggle User.").await))
        }
    }
}

#[get("/<link>")]
pub async fn find(link: String, flash: Option<FlashMessage<'_>>,conn: DbConn) -> Result<Template, Redirect> {
    let flash = flash.map(FlashMessage::into_inner);

    match User::find_with_link(link.clone(), &conn).await {
        Ok(l) => {
            let latest = Ping::get_latest_by_origin(link, &conn).await.unwrap();
            let latest = latest.first();
            Ok(Template::render("sayhi", LinkContext { user: l, flash, latest: latest.cloned() }))
        },
        Err(_) => Err(Redirect::to("/admin"))
    }
}

#[post("/<link>", data = "<thought_form>")]
pub async fn ping(link: String,thought_form: Form<Thought>, conn:DbConn) -> Flash<Redirect> {
    let thought = thought_form.into_inner();
    let latest = Ping::get_latest_by_origin(link.clone(), &conn).await.unwrap();
    let latest = latest.first();


    if ! helpers::valid_hex(&thought.color) {
        return Flash::error(Redirect::to(format!("/user/{}", link)), "That's not a color, wha?");
    }

    if (chrono::Utc::now().naive_utc() - latest.unwrap_or(&Ping::default()).timestamp.unwrap_or_default()) > chrono::Duration::minutes(5)
        || latest.is_none() {
        match Ping::insert(thought.clone(), link.clone(), &conn).await {
            Ok(_) => {
                match helpers::haas_api(thought.color, link.clone()).await {
                    Ok(_) => Flash::success(Redirect::to(format!("/user/{}", link)), "Ping sent"),
                    Err(_) => Flash::error(Redirect::to(format!("/user/{}", link)), "Ping could not be sent due to an Home Assistant API error.")
                }
            },
            Err(e) => {
                error_!("DB insertion error: {}", e);
                Flash::error(Redirect::to(format!("/user/{}", link)), "Ping could not be sent due to an internal error.")
            }
        }
    } else {
        Flash::error(Redirect::to(format!("/user/{}", link)), "Last ping was less than 5 minutes ago, please wait")
    }
}

#[delete("/<id>")]
pub async fn delete(id: i32, conn: DbConn) -> Result<Flash<Redirect>, Template> {
    match User::delete_with_id(id, &conn).await {
        Ok(_) => Ok(Flash::success(Redirect::to("/admin"), "User was deleted.")),
        Err(e) => {
            error_!("DB deletion({}) error: {}", id, e);
            Err(Template::render("index", Context::err(&conn, "Failed to delete user.").await))
        }
    }
}

#[get("/")]
pub async fn index(flash: Option<FlashMessage<'_>>, conn: DbConn) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render("index", Context::raw(&conn, flash).await)
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    flash: Option<(String, String)>,
    users: Vec<User>,
    /// The Ping object itself and the name of Ping origin
    latest_ping: (Option<Ping>, Option<User>)
}


impl Context {
    pub async fn err<M: std::fmt::Display>(conn: &DbConn, msg: M) -> Context {
        let latest= Ping::get_latest(conn).await.unwrap();
        let latest = latest.first();
        Context {
            flash: Some(("error".into(), msg.to_string())),
            users: User::all(conn).await.unwrap_or_default(),
            latest_ping: (latest.cloned(), None)
        }
    }

    pub async fn raw(conn: &DbConn, flash: Option<(String, String)>) -> Context {
        let latest= Ping::get_latest(conn).await.unwrap();
        let latest = latest.first();
        let origin_name = match latest {
            None => {None}
            Some(l) => {User::find_with_link(l.origin.clone().expect("origin"), conn).await.unwrap().first().cloned()}
        };
        match User::all(conn).await {
            Ok(users) => Context { flash, users, latest_ping: (latest.cloned(), origin_name) },
            Err(e) => {
                error_!("DB Task::all() error: {}", e);
                Context {
                    flash: Some(("error".into(), "Fail to access database.".into())),
                    users: vec![],
                    latest_ping: (None, None)
                }
            }
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct LinkContext {
    user: Vec<User>,
    flash: Option<(String, String)>,
    latest: Option<Ping>
}