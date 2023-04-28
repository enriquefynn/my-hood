-- Add migration script here
CREATE TABLE IF NOT EXISTS user(
            id CHAR(36) PRIMARY KEY NOT NULL,
            name VARCHAR(250) NOT NULL,
            birthday DATETIME NOT NULL,
            address VARCHAR(250) NOT NULL,
            activity VARCHAR(250),
            email VARCHAR(250),
            personal_phone VARCHAR(25),
            commercial_phone VARCHAR(25),
            uses_whatsapp BOOLEAN NOT NULL,
            signed_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            identities VARCHAR(1024) NOT NULL
);