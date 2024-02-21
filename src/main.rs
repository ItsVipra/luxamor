// #[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_sync_db_pools;
#[macro_use] extern crate diesel;

mod routes;
mod schema;
mod db_structs;
mod helpers;
mod settings;
mod tests;

use rocket::{Rocket, Build, catchers};

#[database("sqlite_database")]
pub struct DbConn(diesel::SqliteConnection);

#[rocket::launch]
fn rocket() -> _ {
    let config = settings::get_config().expect("unable to read config");

    if config.get_string("PRE_SHARED_KEY").expect("should always have a default value").starts_with("RUNTIME_GENERATED") {
        use inline_colorization::*;
        println!("{color_yellow}{style_bold}Warning:{style_reset} Pre shared key configuration not set!{color_reset}");
        println!("{color_blue}Hint:{style_reset} Your password will be automatically generated each run{color_reset}");
        println!("{color_blue}Help:{style_reset} To avoid this {color_blue}set 'PRE_SHARED_KEY' in luxamor.toml{color_reset}");
        println!("Your password for this run: {color_green}{}{color_reset}", config.get_string("PRE_SHARED_KEY").expect("just passed check"));
    }



    use rocket::fairing::AdHoc;
    use rocket_dyn_templates::Template;
    use rocket::fs::{FileServer, relative};
    use rocket::routes;
    use crate::routes::admin::*;
    use crate::routes::user::*;
    use rocket_governor::rocket_governor_catcher;

    rocket::build()
        .attach(DbConn::fairing())
        .attach(Template::fairing())
        .attach(AdHoc::on_ignite("Run Migrations", run_migrations))
        .mount("/", FileServer::from(relative!("static")))
        .mount("/admin", routes![index, no_auth_index, login, login_page, post_login, logout])
        .mount("/user", routes![new, toggle, delete, find, ping])
        .register("/", catchers!(rocket_governor_catcher))

}

async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

    DbConn::get_one(&rocket).await
        .expect("database connection")
        .run(|conn| { conn.run_pending_migrations(MIGRATIONS).expect("diesel migrations"); })
        .await;

    rocket
}