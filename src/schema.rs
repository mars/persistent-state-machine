table! {
    cryos (id) {
        id -> Int4,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
        state_name -> Text,
        state_data -> Jsonb,
    }
}
