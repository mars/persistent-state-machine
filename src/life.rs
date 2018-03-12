use chrono::prelude::*;
use models::{Life, NewLife};
use diesel;
use diesel::prelude::*;
use diesel::PgConnection;
use r2d2;
use r2d2_diesel::ConnectionManager;
use serde_json;
use schema;

impl Life {
    fn new(db_connection_pool: r2d2::Pool<ConnectionManager<PgConnection>>) -> Self {
        let connection = db_connection_pool.get()
            .expect("get Postgres connection from pool");
        let now = Utc::now().naive_utc();

        let new_life = NewLife {
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
    fn new_as_phase(db_connection_pool: r2d2::Pool<ConnectionManager<PgConnection>>) -> Phase {
        Phase::Gestating(Gestating {
            life: Life::new(db_connection_pool)
        })
    }
}

// Possible states
#[derive(Debug)]
enum Phase {
    Gestating(Gestating),
    Alive(Alive),
    Dead(Dead),
}

// Life state
#[derive(Debug)]
struct Gestating {
    life: Life
}

// Life state
#[derive(Debug)]
struct Alive {
    life: Life
}

// Life state
#[derive(Debug)]
struct Dead {
    life: Life
}

enum Event {
    Birth { occurred_at: DateTime<Utc> },
    Death { occurred_at: DateTime<Utc> },
}

impl From<Gestating> for Alive {
    fn from(val: Gestating) -> Alive {
        let now = Utc::now().naive_utc();
        Alive {
            life: Life {
                updated_at: Some(now),
                born_at: Some(now),
                ..val.life
            },
        }
    }
}

impl From<Alive> for Dead {
    fn from(val: Alive) -> Dead {
        let now = Utc::now().naive_utc();
        Dead {
            life: Life {
                updated_at: Some(now),
                died_at: Some(now),
                ..val.life
            },
        }
    }
}

impl From<Gestating> for Dead {
    fn from(val: Gestating) -> Dead {
        let now = Utc::now().naive_utc();
        Dead {
            life: Life {
                updated_at: Some(now),
                died_at: Some(now),
                ..val.life
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;
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
            life: Life::new(pool)
        });
        match life_phase {
            Phase::Gestating(val) => assert_eq!(val.life.born_at, None),
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

        let life_phase = Life::new_as_phase(pool);
        match life_phase {
            Phase::Gestating(val) => assert_eq!(val.life.born_at, None),
            val => assert!(false, format!("Expecting Gestating; instead got {:?}", val)),
        }
    }

    #[test]
    fn gestating_becomes_alive() {
        let now = Utc::now().naive_utc();
        let gestating = Gestating {
            life: Life {
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
                match val.life.born_at {
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
            life: Life {
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
                match val.life.died_at {
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
            life: Life {
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
                match val.life.died_at {
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

}
