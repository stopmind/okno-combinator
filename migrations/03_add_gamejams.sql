CREATE TABLE gamejams (
    id INT PRIMARY KEY
        GENERATED ALWAYS AS IDENTITY,
    name TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    link TEXT NOT NULL
);

CREATE TABLE community_content (
    id INT PRIMARY KEY
        GENERATED ALWAYS AS IDENTITY,
    name TEXT NOT NULL UNIQUE,
    link TEXT NOT NULL,
    author TEXT NOT NULL,
    label TEXT NOT NULL,
    description TEXT NOT NULL
);

CREATE TABLE gamejam_content (
    gamejam_id INT NOT NULL
         REFERENCES gamejams (id)
         ON DELETE CASCADE,
    content_id INT NOT NULL
         REFERENCES community_content
         ON DELETE CASCADE
);