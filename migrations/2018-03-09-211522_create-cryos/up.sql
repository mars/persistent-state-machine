CREATE TABLE cryos (
  id serial primary key,
  created_at timestamp not null,
  updated_at timestamp,
  state_name text not null,
  state_data jsonb not null
)