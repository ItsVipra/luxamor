use diesel::{QueryResult, RunQueryDsl, prelude::*};
use rocket::serde::Serialize;
use crate::DbConn;
use crate::schema::pings;

#[derive(Serialize, Queryable, Insertable, Debug, Clone, Default)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = pings)]
pub struct Ping {
    pub id: Option<i32>,
    pub timestamp: Option<chrono::NaiveDateTime>,
    pub origin: Option<String>,
    pub color: String
}

// Users should only be able to submit color, nothing else
// the example above implements a secondary struct for that

#[derive(Debug, FromForm, Clone)]
pub struct Thought {
    pub color: String
}

impl Ping {
    pub async fn insert(thought: Thought, origin: String, conn: &DbConn) -> QueryResult<usize> {
        conn.run(move |c| {
            let p = Ping {id: None, origin: Some(origin), timestamp: Some(chrono::Utc::now().naive_utc()), color: thought.color};
            diesel::insert_into(pings::table).values(&p).execute(c)
        }).await
    }

    pub async fn get_latest(conn: &DbConn) -> QueryResult<Vec<Ping>> {
        conn.run(move |c|{
            pings::table.order(pings::timestamp.desc()).limit(1).load::<Ping>(c)
        }).await
    }

    pub async fn get_latest_by_origin(origin: String, conn: &DbConn) -> QueryResult<Vec<Ping>> {
        conn.run(move |c| {
            pings::table.filter(pings::origin.eq(origin)).order(pings::timestamp.desc()).limit(1).load::<Ping>(c)
        }).await
    }

    pub async fn count_by_origin(_origin: String, _conn: &DbConn) -> QueryResult<usize> {
        todo!()
    }
}