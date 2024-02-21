use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::form::Form;
use rocket::serde::Serialize;
use rocket_dyn_templates::Template;
use rocket::{post, get, error_, http, routes};
use rocket::http::Status;

use crate::DbConn;
use crate::db_structs::users::*;
use crate::db_structs::pings::*;
use crate::helpers;

use lazy_static::lazy_static;
lazy_static! {
    static ref CONFIG: config::Config = super::super::settings::get_config().expect("config should have passed checks before");
}

use rocket_governor::{Method, Quota, RocketGovernable, RocketGovernor};

pub struct RateLimitGuard;

impl<'r> RocketGovernable<'r> for RateLimitGuard {
    fn quota(_method: Method, _route_name: &str) -> Quota {
        Quota::per_minute(Self::nonzero(100u32))
    }
}

#[get("/<link>")]
pub async fn find(link: String, flash: Option<FlashMessage<'_>>,conn: DbConn, _limitguard: RocketGovernor<'_, RateLimitGuard>) -> Result<Template, Status> {
    let flash = flash.map(FlashMessage::into_inner);

    match User::find_with_link(link.clone(), &conn).await {
        Ok(l) => {
            if l.is_empty() {
                Err(Status::NotFound)
            } else {
                let latest = Ping::get_latest_by_origin(link, &conn).await.unwrap();
                let latest = latest.first();
                Ok(Template::render("sayhi", LinkContext { user: l, flash, latest: latest.cloned(), meta: (CONFIG.get_int("ping_timeout").unwrap_or(300)*1000, CONFIG.get_string("admin_name").ok()) }))
            }

        },
        Err(_) => Err(http::Status::InternalServerError)
    }
}

#[post("/<link>", data = "<thought_form>")]
pub async fn ping(link: String,thought_form: Form<Thought>, conn:DbConn, _limitguard: RocketGovernor<'_, RateLimitGuard>) -> Result<Flash<Redirect>, (http::Status, &'static str)> {
    let thought = thought_form.into_inner();
    let latest = Ping::get_latest_by_origin(link.clone(), &conn).await.unwrap();
    let latest = latest.first();

    //FIXME: no check for user existing
    if User::find_with_link(link.clone(), &conn).await.unwrap().is_empty() {
        return Err((Status::NotFound, "Invalid link"));
    }

    if !helpers::valid_hex(&thought.color) {
        return Err((Status::BadRequest, "That's not a color, wha? (try #b00b69)")); // Flash::error(Redirect::to(format!("/user/{}", link)), "That's not a color, wha?"));
    }

    if (chrono::Utc::now().naive_utc() - latest.unwrap_or(&Ping::default()).timestamp.unwrap_or_default()) > chrono::Duration::seconds(CONFIG.get_int("ping_timeout").unwrap_or(300))
        || latest.is_none() {
        match Ping::insert(thought.clone(), link.clone(), &conn).await {
            Ok(_) => {
                match helpers::haas_api(thought.color, link.clone()).await {
                    Ok(_) => Ok(Flash::success(Redirect::to(format!("/user/{}", link)), "Ping sent")),
                    Err(_) => Ok(Flash::error(Redirect::to(format!("/user/{}", link)), "Ping could not be sent due to an Home Assistant API error."))
                }
            },
            Err(e) => {
                error_!("DB insertion error: {}", e);
                Ok(Flash::error(Redirect::to(format!("/user/{}", link)), "Ping could not be sent due to an internal error."))
            }
        }
    } else {
        Ok(Flash::error(Redirect::to(format!("/user/{}", link)), format!("Last ping was less than {} minutes ago, please wait", CONFIG.get_int("ping_timeout").unwrap_or(300) / 60)))
    }
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct LinkContext {
    user: Vec<User>,
    flash: Option<(String, String)>,
    latest: Option<Ping>,
    meta: (i64, Option<String>)
}

pub fn routes() -> Vec<rocket::Route> {
    routes![find, ping]
}