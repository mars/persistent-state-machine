table! {
    lives (id) {
        id -> Int4,
        state_type -> Text,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
        born_at -> Nullable<Timestamp>,
        died_at -> Nullable<Timestamp>,
    }
}
