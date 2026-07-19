-- Rename existing columns first to simplify mapping
ALTER TABLE users RENAME COLUMN username TO email;
ALTER TABLE users RENAME COLUMN passhash TO password_hash;

-- Create the new table structure with the correct default for created_at
CREATE TABLE users_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- Copy existing user records
INSERT INTO users_new (id, email, password_hash)
SELECT id, email, password_hash FROM users;

-- Drop the old users table
DROP TABLE users;

-- Rename the new table to users
ALTER TABLE users_new RENAME TO users;
