use schema::lives;
use chrono;

#[derive(Insertable, Debug)]
#[table_name="lives"]
pub struct NewLife {
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub born_at: Option<chrono::NaiveDateTime>,
    pub died_at: Option<chrono::NaiveDateTime>,
}

#[derive(Queryable, Debug)]
pub struct Life {
    pub id: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub born_at: Option<chrono::NaiveDateTime>,
    pub died_at: Option<chrono::NaiveDateTime>,
}