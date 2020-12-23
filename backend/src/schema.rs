table! {
    cells (id) {
        id -> Int4,
        sheet_id -> Int4,
        row -> Int4,
        col -> Int4,
        raw -> Varchar,
    }
}

table! {
    sheets (id) {
        id -> Int4,
    }
}

allow_tables_to_appear_in_same_query!(
    cells,
    sheets,
);
