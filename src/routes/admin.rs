use rocket::request::{FlashMessage, FromRequest, Request};
use rocket::response::{Flash, Redirect};
use rocket::form::Form;
use rocket::serde::Serialize;
use rocket_dyn_templates::Template;
use rocket::{post, put, get, delete, error_, routes, Rocket, Response, http};
use rocket::http::{Status, CookieJar, ContentType};
use rocket::outcome::IntoOutcome;

use crate::DbConn;
use crate::db_structs::users::*;
use crate::db_structs::pings::*;
use crate::helpers;

use lazy_static::lazy_static;
lazy_static! {
    static ref CONFIG: config::Config = super::super::settings::get_config().expect("config should have passed checks before");
}

#[derive(rocket::FromForm)]
pub struct Login<'r> {
    password: &'r str
}

pub struct Admin(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Admin {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> rocket::request::Outcome<Admin, Self::Error> {
        request.cookies()
            .get_private("user_psk")
            .and_then(|cookie| cookie.value().parse().ok())
            .map(Admin)
            .or_forward(Status::Unauthorized)
    }
}


#[get("/")]
pub async fn index(flash: Option<FlashMessage<'_>>, conn: DbConn, _admin: Admin) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render("index", Context::raw(&conn, flash).await)
}

#[get("/", rank = 2)]
pub async fn no_auth_index() -> Redirect {
    Redirect::to("/admin/login")
}

#[get("/login")]
pub fn login(_admin: Admin) -> Redirect {
    Redirect::to("/admin")
}

#[get("/login", rank = 2)]
pub fn login_page(flash: Option<FlashMessage<'_>>) -> Template {
    Template::render("login", LoginContext {flash: flash.map(FlashMessage::into_inner)})
}

use crate::ratelimit::RateLimitGuard;
use rocket_governor::RocketGovernor;
#[post("/login", data = "<login>")]
pub fn post_login(jar: &CookieJar<'_>, login: Form<Login<'_>>, _limitguard: RocketGovernor<'_, RateLimitGuard>) -> Flash<Redirect> {
    let psk = CONFIG.get_string("PRE_SHARED_KEY").expect("pre shared key has to be set");
    if login.password == psk {
        jar.add_private(("user_psk", psk));
        println!("Successful admin login");
        Flash::success(Redirect::to("/admin"), "Successfully logged in")
    } else {
        error_!("Failed admin login attempt, with psk: {}", psk);
        // println!("Failed admin login attempt, with psk: {}", psk);
        Flash::error(Redirect::to("/admin/login"), "Invalid password.")
    }
}

#[post("/logout")]
pub fn logout(jar: &CookieJar<'_>) -> Flash<Redirect> {
    jar.remove_private("user_psk");
    Flash::success(Redirect::to("/admin/login"), "Successfully logged out.")
}

#[post("/", data = "<user_form>")]
pub async fn new(user_form: Form<NewUser>, conn: DbConn, _admin: Admin) -> Flash<Redirect> {
    let newuser = user_form.into_inner();
    if newuser.name.is_empty() {
        Flash::error(Redirect::to("/admin"), "Name cannot be empty.")
    } else if let Err(e) = User::insert(newuser, helpers::new_link(CONFIG.get_int("link_len").unwrap_or(16) as usize), &conn).await {
        error_!("DB insertion error: {}", e);
        Flash::error(Redirect::to("/admin"), "User could not be inserted due an internal error.")
    } else {
        Flash::success(Redirect::to("/admin"), "User successfully added.")
    }
}

#[put("/<id>")]
pub async fn toggle(id: i32, conn: DbConn, _admin: Admin) -> Result<Redirect, Template> {
    match User::toggle_with_id(id, &conn).await {
        Ok(_) => Ok(Redirect::to("/admin")),
        Err(e) => {
            error_!("DB toggle({}) error: {}", id, e);
            Err(Template::render("index", Context::err(&conn, "Failed to toggle User.").await))
        }
    }
}

#[delete("/<id>")]
pub async fn delete(id: i32, conn: DbConn, _admin: Admin) -> Result<Flash<Redirect>, Template> {
    match User::delete_with_id(id, &conn).await {
        Ok(_) => Ok(Flash::success(Redirect::to("/admin"), "User was deleted.")),
        Err(e) => {
            error_!("DB deletion({}) error: {}", id, e);
            Err(Template::render("index", Context::err(&conn, "Failed to delete user.").await))
        }
    }
}

#[get("/latest")]
pub async fn latest(conn: DbConn, _admin: Admin, _limitguard: RocketGovernor<'_, RateLimitGuard>) -> Result<(Status, rocket::serde::json::Json<Ping>), http::Status> {
    let latest= Ping::get_latest(&conn).await.unwrap();
    match latest.first() {
        Some(p) => Ok((Status::Ok, rocket::serde::json::Json(p.clone()))),
        None => Err(Status::NotFound)
    }
}


#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    flash: Option<(String, String)>,
    users: Vec<User>,
    /// The Ping object itself and the name of Ping origin
    latest_ping: (Option<Ping>, Option<User>)
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct LoginContext {
    flash: Option<(String, String)>
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
            Some(l) => { User::find_with_link(l.origin.clone().expect("origin"), conn).await.unwrap().first().cloned()}
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

pub fn routes() -> Vec<rocket::Route> {
    routes![index, new, toggle, delete]
}