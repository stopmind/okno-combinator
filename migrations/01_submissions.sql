CREATE TYPE submission_status AS ENUM ('unverified', 'pending', 'reviewed');

CREATE TABLE submissions (
    id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    status submission_status NOT NULL,
    name TEXT NOT NULL,
    link TEXT NOT NULL,
    date DATE NOT NULL DEFAULT (CURRENT_DATE),
    author TEXT NOT NULL,
    additional_info TEXT
);

CREATE TABLE tag_categories (
    id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    title TEXT UNIQUE NOT NULL,
    panel_only BOOLEAN NOT NULL,
    "order" INT NOT NULL
);

CREATE TABLE tags (
    id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    category_id INT NOT NULL
        REFERENCES tag_categories (id) ON DELETE CASCADE ,
    title TEXT UNIQUE NOT NULL
);

CREATE TABLE submission_tag (
    submission_id INT NOT NULL
        REFERENCES submissions (id) ON DELETE CASCADE,
    tag_id INT NOT NULL
        REFERENCES tags (id) ON DELETE CASCADE,

    PRIMARY KEY (submission_id, tag_id)
);

