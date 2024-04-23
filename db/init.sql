CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) UNIQUE NOT NULL
);

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(100) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password VARCHAR(255) NOT NULL,
    salt VARCHAR(255) NOT NULL,
    role_id UUID REFERENCES roles(id)
);

INSERT INTO roles (name) VALUES ('server');
INSERT INTO roles (name) VALUES ('client');

INSERT INTO users (username, email, password, salt, role_id)
VALUES ('server', 'server@uqac.ca', 'hashed_password', 'salt_value', (SELECT id FROM roles WHERE name = 'server'));


