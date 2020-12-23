CREATE TABLE haikus (
    id BIGSERIAL,
    channel BIGINT NOT NULL,
    server BIGINT NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    author_0 BIGINT NOT NULL,
    author_1 BIGINT NOT NULL,
    author_2 BIGINT NOT NULL,
    message_0 TEXT NOT NULL,
    message_1 TEXT NOT NULL,
    message_2 TEXT NOT NULL,
    PRIMARY KEY (id, server)
);

