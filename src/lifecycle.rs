use chrono::prelude::*;
use diesel::PgConnection;
use std::mem;

use models::Life;

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
    pub fn create(db_connection: &PgConnection) -> Phase {
        let life = Life::create(db_connection);
        life.as_phase()
    }
    pub fn find_by_life(db_connection: &PgConnection, id: i32) -> Phase {
        let life = Life::find(db_connection, id);
        life.as_phase()
    }
    pub fn save(&mut self, db_connection: &PgConnection) -> &mut Self {
        let new_self = self
            .as_life()
            .save(db_connection)
            .as_phase();
        mem::replace(self, new_self);
        self
    }
    pub fn as_life(&self) -> Life {
        let life = match self {
            &Phase::Gestating(ref v) => &v.state,
            &Phase::Alive(ref v) => &v.state,
            &Phase::Dead(ref v) => &v.state,
        };
        life.to_owned()
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
        let connection = pool.get()
            .expect("get Postgres connection from pool");

        let life_phase = Phase::Gestating(Gestating {
            state: Life::create(&connection)
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
        let connection = pool.get()
            .expect("get Postgres connection from pool");

        let life_phase = Phase::create(&connection);
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
        let connection = pool.get()
            .expect("get Postgres connection from pool");

        let gestating = if let Phase::Gestating(p) = Phase::create(&connection) { p }
            else { panic!("Not Gestating") };

        let result = Phase::find_by_life(&connection, gestating.state.id);
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
        let connection = pool.get()
            .expect("get Postgres connection from pool");

        let phase = Phase::create(&connection);
        let gestating = if let Phase::Gestating(p) = phase { p }
            else { panic!("Not Gestating") };

        let mut alive = Phase::Alive(gestating.into());
        alive.save(&connection);

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
        let connection = pool.get()
            .expect("get Postgres connection from pool");

        let phase = Phase::create(&connection);
        let gestating = if let Phase::Gestating(p) = phase { p }
            else { panic!("Not Gestating") };

        let mut dead = Phase::Dead(gestating.into());
        dead.save(&connection);

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
