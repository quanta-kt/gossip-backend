CREATE TABLE gossip_user(
    id SERIAL PRIMARY KEY NOT NULL,
    username TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,

    is_verified BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE pending_email_verification(
    user_id INTEGER PRIMARY KEY REFERENCES gossip_user(id),
    code TEXT NOT NULL
)
