table! {
    users {
        id -> Nullable<Integer>,
        name -> Text,
        link -> Nullable<Text>,
        enabled -> Bool,
    }
}

table!{
    pings {
        id -> Nullable<Integer>,
        timestamp -> Nullable<Timestamp>,
        origin -> Nullable<Text>,
        color -> Text
    }
}