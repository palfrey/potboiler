table! {
    use diesel::sql_types::*;
    logs {
        id -> Uuid,
        owner -> Uuid,
        next -> Nullable<Uuid>,
        prev -> Nullable<Uuid>,
        data -> Jsonb,
        hlc_tstamp -> Binary,
    }
}

table! {
    nodes(url) {
        url -> VarChar,
    }
}