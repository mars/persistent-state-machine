use chrono::prelude::*;
use diesel;
use diesel::prelude::*;
use diesel::PgConnection;
use r2d2;
use r2d2_diesel::ConnectionManager;
use std::mem;

use models::Life;
use schema;

pub const STATE_NAME_GESTATING: &'static str = "Gestating";
pub const STATE_NAME_ALIVE: &'static str = "Alive";
pub const STATE_NAME_DEAD: &'static str = "Dead";

// Possible states
#[derive(Debug)]
pub enum Phase {
    Gestating(Gestating),
    Alive(Alive),
    Dead(Dead),
}

impl Phase {
    pub fn create(db_connection_pool: &r2d2::Pool<ConnectionManager<PgConnection>>) -> Phase {
        let life = Life::create(db_connection_pool);
        Phase::Gestating(Gestating { state: life })
    }
    pub fn find_by_life(db_connection_pool: &r2d2::Pool<ConnectionManager<PgConnection>>, id: i32) -> Phase {
        let connection = db_connection_pool.get()
            .expect("get Postgres connection from pool");
        let life_result = schema::lives::table
            .filter(schema::lives::columns::id.eq(id))
            .get_result::<Life>(&*connection);
        let life = match life_result {
            Ok(v) => v,
            Err(e) => panic!("Error finding database record (lives.id: {:?}): {:?})", id, e),
        };
        life.as_phase()
    }
    pub fn save(&mut self, db_connection_pool: &r2d2::Pool<ConnectionManager<PgConnection>>) -> () {
        let mut life = match *self {
            Phase::Gestating(ref v) => v.state.to_owned(),
            Phase::Alive(ref v) => v.state.to_owned(),
            Phase::Dead(ref v) => v.state.to_owned(),
        };
        life.save(db_connection_pool);
        let new_self = life.as_phase();
        mem::replace(self, new_self);
    }
}

// Life state
#[derive(Debug)]
pub struct Gestating {
    pub state: Life
}

// Life state
#[derive(Debug)]
pub struct Alive {
    pub state: Life
}

// Life state
#[derive(Debug)]
pub struct Dead {
    pub state: Life
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
            state: Life::create(&pool)
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

        let life_phase = Phase::create(&pool);
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

        let gestating = if let Phase::Gestating(p) = Phase::create(&pool) { p }
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

        let phase = Phase::create(&pool);
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

        let phase = Phase::create(&pool);
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
