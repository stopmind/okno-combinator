CREATE TABLE users (
    id INT PRIMARY KEY
       GENERATED ALWAYS AS IDENTITY,
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    locked BOOLEAN NOT NULL
        DEFAULT (false),
    admin BOOLEAN NOT NULL
);