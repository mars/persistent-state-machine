use chrono;
use chrono::prelude::*;
use diesel;
use diesel::prelude::*;
use diesel::PgConnection;
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;
use std::mem;

use lifecycle::{STATE_NAME_GESTATING, STATE_NAME_ALIVE, STATE_NAME_DEAD, Phase, Gestating, Alive, Dead};
use schema::lives;

#[derive(Insertable, Debug)]
#[table_name="lives"]
pub struct NewLife {
    pub state_type: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub born_at: Option<chrono::NaiveDateTime>,
    pub died_at: Option<chrono::NaiveDateTime>,
}

#[derive(Queryable, AsChangeset, Debug, Clone)]
#[table_name="lives"]
pub struct Life {
    pub id: i32,
    pub state_type: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub born_at: Option<chrono::NaiveDateTime>,
    pub died_at: Option<chrono::NaiveDateTime>,
}

impl Life {
    pub fn create(db_connection_pool: &Pool<ConnectionManager<PgConnection>>) -> Self {
        let connection = db_connection_pool.get()
            .expect("get Postgres connection from pool");
        let now = Utc::now().naive_utc();

        let new_life = NewLife {
            state_type: String::from(STATE_NAME_GESTATING),
            created_at: now,
            updated_at: Some(now),
            born_at: None,
            died_at: None,
        };

        let database_record = diesel::insert_into(lives::table)
            .values(&new_life)
            .get_result::<Life>(&*connection)
            .expect("Error creating new Life");

        database_record
    }
    pub fn find(db_connection_pool: &Pool<ConnectionManager<PgConnection>>, id: i32) -> Self {
        let connection = db_connection_pool.get()
            .expect("get Postgres connection from pool");
        let life_result = lives::table
            .filter(lives::columns::id.eq(id))
            .get_result::<Life>(&*connection);
        match life_result {
            Ok(v) => v,
            Err(e) => panic!("Error finding database record (lives.id: {:?}): {:?})", id, e),
        }
    }
    pub fn save(&mut self, db_connection_pool: &Pool<ConnectionManager<PgConnection>>) -> () {
        let connection = db_connection_pool.get()
            .expect("get Postgres connection from pool");
        let now = Utc::now().naive_utc();
        let updated_life = Life {
            updated_at: Some(now),
            ..self.to_owned()
        };
        let life_result = diesel::update(lives::table)
            .set(&updated_life)
            .get_result::<Life>(&*connection);
        let new_self = match life_result {
            Ok(v) => v,
            Err(e) => panic!("Error updating database record (lives.id: {:?}): {:?})", self.id, e),
        };
        mem::replace(self, new_self);
    }
    pub fn as_phase(&self) -> Phase {
        let life = self.to_owned();
        match life.state_type.clone().as_ref() {
            STATE_NAME_GESTATING    => Phase::Gestating(Gestating { state: life }),
            STATE_NAME_ALIVE        => Phase::Alive(Alive { state: life }),
            STATE_NAME_DEAD         => Phase::Dead(Dead { state: life }),
            invalid_name            => panic!(
                "Invalid state name (state_type: {:?}) found in database record (id: {:?})",
                invalid_name, self.id),
        }
    }
}