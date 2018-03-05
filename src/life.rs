extern crate chrono;
use self::chrono::prelude::*;

#[derive(Debug)]
struct Life<P> {
    state: P,
}
impl Life<Gestating> {
    fn new() -> Self {
        Life {
            state: Gestating {},
        }
    }
    fn new_as_phase() -> Phase {
        Phase::Gestating(Life::new())
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
#[derive(Debug)]
struct Gestating;

// Life state
#[derive(Debug)]
struct Alive {
    born_at: DateTime<Utc>,
}

// Life state
#[derive(Debug)]
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
    use super::chrono::prelude::*;

    #[test]
    fn starts_gestating_as_life() {
        let life_phase = Phase::Gestating(Life::new());
        match life_phase {
            Phase::Gestating(val) => assert!(true),
            val => assert!(false, format!("Expecting Gestating; instead got {:?}", val)),
        }
    }

    #[test]
    fn starts_gestating_as_phase() {
        let life_phase = Life::new_as_phase();
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
