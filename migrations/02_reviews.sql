CREATE TABLE reviews (
    id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    submission_id INT NOT NULL
        REFERENCES submissions (id) ON DELETE CASCADE,
    author TEXT NOT NULL,
    link TEXT NOT NULL,
    rating INT NOT NULL
);