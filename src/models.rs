use schema::cryos;
use chrono;
use serde_json;

#[derive(Insertable, Debug)]
#[table_name="cryos"]
pub struct NewCryo {
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub state_name: String,
    pub state_data: serde_json::Value,
}

#[derive(Queryable, Debug)]
pub struct Cryo {
    pub id: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub state_name: String,
    pub state_data: serde_json::Value,
}