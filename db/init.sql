CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) UNIQUE NOT NULL
);

CREATE TABLE ranks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    image_url VARCHAR(255)
);

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(100) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password VARCHAR(255) NOT NULL,
    salt VARCHAR(255) NOT NULL,
    kda FLOAT DEFAULT 0,
    nb_games INT DEFAULT 0,
    role_id UUID REFERENCES roles(id),
    rank_id UUID REFERENCES ranks(id)
);

CREATE TABLE achievements (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    image_url VARCHAR(255)
);

CREATE TABLE user_achievements (
    user_id UUID REFERENCES users(id),
    achievement_id UUID REFERENCES achievements(id),
    PRIMARY KEY (user_id, achievement_id)
);

CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    average_kda FLOAT,
    average_rank UUID REFERENCES ranks(id),
    is_empty BOOLEAN DEFAULT TRUE
);

CREATE TABLE friends (
    user_id UUID REFERENCES users(id),
    friend_id UUID REFERENCES users(id),
    PRIMARY KEY (user_id, friend_id)
);

CREATE Table friend_requests (
    user_id UUID REFERENCES users(id),
    friend_id UUID REFERENCES users(id),
    PRIMARY KEY (user_id, friend_id)
);

INSERT INTO roles (name) VALUES ('server');
INSERT INTO roles (name) VALUES ('client');

INSERT INTO ranks (name, image_url) VALUES ('Bronze', 'https://via.placeholder.com/150');
INSERT INTO ranks (name, image_url) VALUES ('Silver', ' https://via.placeholder.com/150');
INSERT INTO ranks (name, image_url) VALUES ('Gold', 'https://via.placeholder.com/150');
INSERT INTO ranks (name, image_url) VALUES ('Platinum', 'https://via.placeholder.com/150');
INSERT INTO ranks (name, image_url) VALUES ('Diamond', 'https://via.placeholder.com/150');

INSERT INTO users (username, email, password, salt, role_id, rank_id)
VALUES ('server', 'server@uqac.ca', '$2y$12$zcm/bCwARboBGYvyTm.89u9G2qhqZL4Bm3ZKMkCI5G59P1/hS1geC', 'd5ea01744c824dda8321ac7456803eff', (SELECT id FROM roles WHERE name = 'server'),(SELECT id FROM ranks WHERE name = 'Bronze'));

INSERT INTO achievements (name, description, image_url) VALUES ('First connection', 'You have successfully connected to the server for the first time.', 'https://via.placeholder.com/150');
