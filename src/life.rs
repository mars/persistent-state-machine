use chrono::prelude::*;
use models::{Cryo, NewCryo};
use diesel;
use diesel::prelude::*;
use diesel::PgConnection;
use r2d2;
use r2d2_diesel::ConnectionManager;
use serde_json;
use schema;

#[derive(Debug)]
struct Life<P> {
    state: P,
}
impl Life<Gestating> {
    fn new(db_connection_pool: r2d2::Pool<ConnectionManager<PgConnection>>) -> Self {
        let connection = db_connection_pool.get().expect("get Postgres connection from pool");
        let now = Utc::now().naive_utc();

        let d: serde_json::Value = serde_json::from_str("{}").unwrap();

        let new_cryo = NewCryo {
            created_at: now,
            updated_at: Some(now),
            state_name: String::from("Gestating"),
            state_data: d,
        };

        let database_record = diesel::insert_into(schema::cryos::table)
            .values(&new_cryo)
            .get_result::<Cryo>(&*connection)
            .expect("Error saving new Cryo");

        Life {
            state: database_record.state_data,
        }
    }
    fn new_as_phase(db_connection_pool: r2d2::Pool<ConnectionManager<PgConnection>>) -> Phase {
        Phase::Gestating(Life::new(db_connection_pool))
    }
}

// Possible states
#[derive(Debug)]
enum Phase {
    Gestating(Life<Gestating>),
    Alive(Life<Alive>),
    Dead(Life<Dead>),
}

// Life state
#[derive(Clone, Serialize, Deserialize, Debug)]
struct Gestating;

// Life state
#[derive(Clone, Serialize, Deserialize, Debug)]
struct Alive {
    born_at: DateTime<Utc>,
}

// Life state
#[derive(Clone, Serialize, Deserialize, Debug)]
struct Dead {
    born_at: Option<DateTime<Utc>>,
    died_at: DateTime<Utc>,
}

enum Event {
    Birth { occurred_at: DateTime<Utc> },
    Death { occurred_at: DateTime<Utc> },
}

impl From<Life<Gestating>> for Life<Alive> {
    fn from(val: Life<Gestating>) -> Life<Alive> {
        Life {
            state: Alive {
                born_at: Utc::now(),
            },
        }
    }
}

impl From<Life<Alive>> for Life<Dead> {
    fn from(val: Life<Alive>) -> Life<Dead> {
        Life {
            state: Dead {
                born_at: Some(val.state.born_at),
                died_at: Utc::now(),
            },
        }
    }
}

impl From<Life<Gestating>> for Life<Dead> {
    fn from(val: Life<Gestating>) -> Life<Dead> {
        Life {
            state: Dead {
                born_at: None,
                died_at: Utc::now(),
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

        let life_phase = Phase::Gestating(Life::new(pool));
        match life_phase {
            Phase::Gestating(val) => assert!(true),
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
            Phase::Gestating(val) => assert!(true),
            val => assert!(false, format!("Expecting Gestating; instead got {:?}", val)),
        }
    }

    #[test]
    fn gestating_becomes_alive() {
        let life = Life {
            state: Gestating {},
        };
        let life_phase = Phase::Alive(life.into());
        match life_phase {
            Phase::Alive(val) => assert!(true),
            val => assert!(false, format!("Expecting Alive; instead got {:?}", val)),
        }
    }

    #[test]
    fn alive_becomes_dead() {
        let life = Life {
            state: Alive {
                born_at: Utc::now(),
            },
        };
        let life_phase = Phase::Dead(life.into());
        match life_phase {
            Phase::Dead(val) => assert!(true),
            val => assert!(false, format!("Expecting Dead; instead got {:?}", val)),
        }
    }

    #[test]
    fn gestating_becomes_dead() {
        let life = Life {
            state: Gestating {},
        };
        let life_phase = Phase::Dead(life.into());
        match life_phase {
            Phase::Dead(val) => assert!(true),
            val => assert!(false, format!("Expecting Dead; instead got {:?}", val)),
        }
    }

}
