-- Create post table.
CREATE TABLE IF NOT EXISTS post (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    author_id INTEGER NOT NULL,
    content TEXT NOT NULL
)
