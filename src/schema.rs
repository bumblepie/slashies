table! {
    haikus (id, server) {
        id -> Int8,
        channel -> Int8,
        server -> Int8,
        timestamp -> Timestamp,
        author_0 -> Int8,
        author_1 -> Int8,
        author_2 -> Int8,
        message_0 -> Text,
        message_1 -> Text,
        message_2 -> Text,
    }
}
