use schema::lives;
use chrono;

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