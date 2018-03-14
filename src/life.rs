use chrono::prelude::*;
use models::{Life, NewLife};
use diesel;
use diesel::prelude::*;
use diesel::PgConnection;
use r2d2;
use r2d2_diesel::ConnectionManager;
use schema;
use std::mem;

const STATE_NAME_GESTATING: &'static str = "Gestating";
const STATE_NAME_ALIVE: &'static str = "Alive";
const STATE_NAME_DEAD: &'static str = "Dead";

impl Life {
    fn new(db_connection_pool: &r2d2::Pool<ConnectionManager<PgConnection>>) -> Self {
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

        let database_record = diesel::insert_into(schema::lives::table)
            .values(&new_life)
            .get_result::<Life>(&*connection)
            .expect("Error saving new Life");

        database_record
    }
}

// Possible states
#[derive(Debug)]
enum Phase {
    Gestating(Gestating),
    Alive(Alive),
    Dead(Dead),
}

impl Phase {
    fn new(db_connection_pool: &r2d2::Pool<ConnectionManager<PgConnection>>) -> Phase {
        let life = Life::new(db_connection_pool);
        Phase::Gestating(Gestating { state: life })
    }
    fn find_by_life(db_connection_pool: &r2d2::Pool<ConnectionManager<PgConnection>>, id: i32) -> Phase {
        let connection = db_connection_pool.get()
            .expect("get Postgres connection from pool");
        let life_result = schema::lives::table
            .filter(schema::lives::columns::id.eq(id))
            .get_result::<Life>(&*connection);
        let life = match life_result {
            Ok(v) => v,
            Err(e) => panic!("Error finding database record (lives.id: {:?}): {:?})", id, e),
        };
        match life.state_type.clone().as_ref() {
            STATE_NAME_GESTATING    => Phase::Gestating(Gestating { state: life }),
            STATE_NAME_ALIVE        => Phase::Alive(Alive { state: life }),
            STATE_NAME_DEAD         => Phase::Dead(Dead { state: life }),
            invalid_name            => panic!(
                "Invalid state name (state_type: {:?}) found in database record (lives.id: {:?})",
                invalid_name, id),
        }
    }
    fn save(&mut self, db_connection_pool: &r2d2::Pool<ConnectionManager<PgConnection>>) -> () {
        let connection = db_connection_pool.get()
            .expect("get Postgres connection from pool");
        let now = Utc::now().naive_utc();
        let life = match *self {
            Phase::Gestating(ref v) => v.state.to_owned(),
            Phase::Alive(ref v) => v.state.to_owned(),
            Phase::Dead(ref v) => v.state.to_owned(),
        };
        let updated_life = Life {
            updated_at: Some(now),
            ..life
        };
        let life_result = diesel::update(schema::lives::table)
            .set(&updated_life)
            .get_result::<Life>(&*connection);
        let life = match life_result {
            Ok(v) => v,
            Err(e) => panic!("Error updating database record (lives.id: {:?}): {:?})", life.id, e),
        };
        let new_self = match life.state_type.clone().as_ref() {
            STATE_NAME_GESTATING    => Phase::Gestating(Gestating { state: life }),
            STATE_NAME_ALIVE        => Phase::Alive(Alive { state: life }),
            STATE_NAME_DEAD         => Phase::Dead(Dead { state: life }),
            invalid_name            => panic!(
                "Invalid state name (state_type: {:?}) after updating database record (lives.id: {:?})",
                invalid_name, life.id),
        };
        mem::replace(self, new_self);
    }
}

// Life state
#[derive(Debug)]
struct Gestating {
    state: Life
}

// Life state
#[derive(Debug)]
struct Alive {
    state: Life
}

// Life state
#[derive(Debug)]
struct Dead {
    state: Life
}

enum Event {
    Birth { occurred_at: DateTime<Utc> },
    Death { occurred_at: DateTime<Utc> },
}

impl From<Gestating> for Alive {
    fn from(val: Gestating) -> Alive {
        let now = Utc::now().naive_utc();
        Alive {
            state: Life {
                state_type: String::from(STATE_NAME_ALIVE),
                updated_at: Some(now),
                born_at: Some(now),
                ..val.state
            },
        }
    }
}

impl From<Alive> for Dead {
    fn from(val: Alive) -> Dead {
        let now = Utc::now().naive_utc();
        Dead {
            state: Life {
                state_type: String::from(STATE_NAME_DEAD),
                updated_at: Some(now),
                died_at: Some(now),
                ..val.state
            },
        }
    }
}

impl From<Gestating> for Dead {
    fn from(val: Gestating) -> Dead {
        let now = Utc::now().naive_utc();
        Dead {
            state: Life {
                state_type: String::from(STATE_NAME_DEAD),
                updated_at: Some(now),
                died_at: Some(now),
                ..val.state
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::PgConnection;
    use dotenv::dotenv;
    use r2d2;
    use r2d2_diesel::ConnectionManager;
    use std::env;

    #[test]
    fn starts_gestating_as_life() {
        dotenv().ok();
        let postgres_url = env::var("DATABASE_URL")
            .expect("requires DATABASE_URL env var");
        let manager = ConnectionManager::<PgConnection>::new(postgres_url);
        let pool = r2d2::Pool::builder().build(manager)
            .expect("Failed to create pool.");

        let life_phase = Phase::Gestating(Gestating {
            state: Life::new(&pool)
        });
        match life_phase {
            Phase::Gestating(val) => assert_eq!(val.state.born_at, None),
            val => assert!(false, format!("Expecting Gestating; instead got {:?}", val)),
        }
    }

    #[test]
    fn starts_gestating_as_phase() {
        dotenv().ok();
        let postgres_url = env::var("DATABASE_URL")
            .expect("requires DATABASE_URL env var");
        let manager = ConnectionManager::<PgConnection>::new(postgres_url);
        let pool = r2d2::Pool::builder().build(manager)
            .expect("Failed to create pool.");

        let life_phase = Phase::new(&pool);
        match life_phase {
            Phase::Gestating(val) => assert_eq!(val.state.born_at, None),
            val => assert!(false, format!("Expecting Gestating; instead got {:?}", val)),
        }
    }

    #[test]
    fn gestating_becomes_alive() {
        let now = Utc::now().naive_utc();
        let gestating = Gestating {
            state: Life {
                state_type: String::from(STATE_NAME_GESTATING),
                id: 42,
                created_at: now,
                updated_at: Some(now),
                born_at: None,
                died_at: None,
            }
        };
        let result = Phase::Alive(gestating.into());
        match result {
            Phase::Alive(val) => {
                match val.state.born_at {
                    Some(at) => {
                        let dur = now.signed_duration_since(at);
                        assert!(dur.num_seconds() <= 1, "Expecting current born_at when Alive")
                    }
                    None => assert!(false, "Expecting born_at timestamp; instead got None")
                }
            },
            val => assert!(false, format!("Expecting Alive; instead got {:?}", val)),
        }
    }

    #[test]
    fn alive_becomes_dead() {
        let now = Utc::now().naive_utc();
        let alive = Alive {
            state: Life {
                state_type: String::from(STATE_NAME_ALIVE),
                id: 42,
                created_at: now,
                updated_at: Some(now),
                born_at: Some(now),
                died_at: None,
            }
        };
        let result = Phase::Dead(alive.into());
        match result {
            Phase::Dead(val) => {
                match val.state.died_at {
                    Some(at) => {
                        let dur = now.signed_duration_since(at);
                        assert!(dur.num_seconds() <= 1, "Expecting current died_at when Dead")
                    }
                    None => assert!(false, "Expecting died_at timestamp; instead got None")
                }
            },
            val => assert!(false, format!("Expecting Dead; instead got {:?}", val)),
        }
    }

    #[test]
    fn gestating_becomes_dead() {
        let now = Utc::now().naive_utc();
        let gestating = Gestating {
            state: Life {
                state_type: String::from(STATE_NAME_GESTATING),
                id: 42,
                created_at: now,
                updated_at: Some(now),
                born_at: Some(now),
                died_at: None,
            }
        };
        let result = Phase::Dead(gestating.into());
        match result {
            Phase::Dead(val) => {
                match val.state.died_at {
                    Some(at) => {
                        let dur = now.signed_duration_since(at);
                        assert!(dur.num_seconds() <= 1, "Expecting current died_at when Dead")
                    }
                    None => assert!(false, "Expecting died_at timestamp; instead got None")
                }
            },
            val => assert!(false, format!("Expecting Dead; instead got {:?}", val)),
        }
    }

    #[test]
    fn finds_phase_by_life() {
        dotenv().ok();
        let postgres_url = env::var("DATABASE_URL")
            .expect("requires DATABASE_URL env var");
        let manager = ConnectionManager::<PgConnection>::new(postgres_url);
        let pool = r2d2::Pool::builder().build(manager)
            .expect("Failed to create pool.");

        let gestating = if let Phase::Gestating(p) = Phase::new(&pool) { p }
            else { panic!("Not Gestating") };

        let result = Phase::find_by_life(&pool, gestating.state.id);
        match result {
            Phase::Gestating(val) => assert_eq!(val.state.born_at, None),
            val => assert!(false, format!("Expecting Gestating; instead got {:?}", val)),
        }
    }

    #[test]
    fn finds_alive_phase_by_life() {
        dotenv().ok();
        let now = Utc::now().naive_utc();

        let postgres_url = env::var("DATABASE_URL")
            .expect("requires DATABASE_URL env var");
        let manager = ConnectionManager::<PgConnection>::new(postgres_url);
        let pool = r2d2::Pool::builder().build(manager)
            .expect("Failed to create pool.");

        let phase = Phase::new(&pool);
        let gestating = if let Phase::Gestating(p) = phase { p }
            else { panic!("Not Gestating") };

        let mut alive = Phase::Alive(gestating.into());
        alive.save(&pool);

        match alive {
            Phase::Alive(val) => {
                match val.state.born_at {
                    Some(at) => {
                        let dur = now.signed_duration_since(at);
                        assert!(dur.num_seconds() <= 1, "Expecting current born_at when Alive")
                    }
                    None => assert!(false, "Expecting born_at timestamp; instead got None")
                }
            },
            val => assert!(false, format!("Expecting Alive; instead got {:?}", val)),
        }
    }

    #[test]
    fn finds_dead_phase_by_life() {
        dotenv().ok();
        let now = Utc::now().naive_utc();

        let postgres_url = env::var("DATABASE_URL")
            .expect("requires DATABASE_URL env var");
        let manager = ConnectionManager::<PgConnection>::new(postgres_url);
        let pool = r2d2::Pool::builder().build(manager)
            .expect("Failed to create pool.");

        let phase = Phase::new(&pool);
        let gestating = if let Phase::Gestating(p) = phase { p }
            else { panic!("Not Gestating") };

        let mut dead = Phase::Dead(gestating.into());
        dead.save(&pool);

        match dead {
            Phase::Dead(val) => {
                match val.state.born_at {
                    Some(at) => {
                        let dur = now.signed_duration_since(at);
                        assert!(dur.num_seconds() <= 1, "Expecting current born_at when Dead")
                    }
                    None => assert!(false, "Expecting born_at timestamp; instead got None")
                }
            },
            val => assert!(false, format!("Expecting Dead; instead got {:?}", val)),
        }
    }

}
