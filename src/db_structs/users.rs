use diesel::{QueryResult, RunQueryDsl, prelude::*};
use crate::DbConn;
use crate::schema::users;
use rocket::serde::Serialize;

#[derive(Serialize, Queryable, Insertable, Debug, Clone)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = users)]
pub struct User {
    #[serde(skip_deserializing)]
    pub id: Option<i32>,
    pub name: String,
    pub link: Option<String>,
    pub enabled: bool
}

#[derive(Debug, FromForm)]
pub struct NewUser {
    pub name: String,
}

impl User {
    pub async fn all(conn: &DbConn) -> QueryResult<Vec<User>> {
        conn.run(|c| {
            users::table.order(users::id.desc()).load::<User>(c)
        }).await
    }

    /// Returns the number of affected rows: 1.
    pub async fn insert(newuser: NewUser, link: Box<str>, conn: &DbConn) -> QueryResult<usize> {
        conn.run(move|c| {
            let t = User { id: None, name: newuser.name, link: Some(link.to_string()), enabled: true };
            diesel::insert_into(users::table).values(&t).execute(c)
        }).await
    }

    /// Returns the number of affected rows: 1.
    pub async fn toggle_with_id(id: i32, conn: &DbConn) -> QueryResult<usize> {
        conn.run(move |c| {
            let user = users::table.filter(users::id.eq(id)).get_result::<User>(c)?;
            let new_status = !user.enabled;
            let updated_user = diesel::update(users::table.filter(users::id.eq(id)));
            updated_user.set(users::enabled.eq(new_status)).execute(c)
        }).await
    }

    pub async fn find_with_link(link: String, conn: &DbConn) -> QueryResult<Vec<User>> {
        conn.run(move |c| {
            users::table.filter(users::link.eq(link)).load::<User>(c)
        }).await
    }

    /// Returns the number of affected rows: 1.
    pub async fn delete_with_id(id: i32, conn: &DbConn) -> QueryResult<usize> {
        conn.run(move |c| diesel::delete(users::table)
            .filter(users::id.eq(id))
            .execute(c))
            .await
    }

    /// Returns the number of affected rows.
    #[cfg(test)]
    pub async fn delete_all(conn: &DbConn) -> QueryResult<usize> {
        conn.run(|c| diesel::delete(users::table).execute(c)).await
    }
}