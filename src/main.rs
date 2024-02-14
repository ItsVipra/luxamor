#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_sync_db_pools;
#[macro_use] extern crate diesel;

mod routes;
mod schema;
mod db_structs;
mod helpers;

use rocket::{Rocket, Build};


#[database("sqlite_database")]
pub struct DbConn(diesel::SqliteConnection);

#[launch]
fn rocket() -> _ {
    use rocket::fairing::AdHoc;
    use rocket_dyn_templates::Template;
    use rocket::fs::{FileServer, relative};
    use crate::routes::*;

    rocket::build()
        .attach(DbConn::fairing())
        .attach(Template::fairing())
        .attach(AdHoc::on_ignite("Run Migrations", run_migrations))
        .mount("/", FileServer::from(relative!("static")))
        .mount("/admin", routes![index])
        .mount("/user", routes![new, toggle, delete, find, ping])

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