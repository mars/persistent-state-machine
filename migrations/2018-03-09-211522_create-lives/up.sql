CREATE TABLE lives (
  id serial primary key,
  state_type text not null,
  created_at timestamp not null,
  updated_at timestamp,
  born_at timestamp,
  died_at timestamp
)