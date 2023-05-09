#!/bin/bash

# Set the path to the SQLite3 database file
DB_PATH="assets/sprites.db"

# Create the spritesheets table
sqlite3 "$DB_PATH" <<EOF
CREATE TABLE spritesheets (
    id INTEGER PRIMARY KEY,
    file_path TEXT UNIQUE,
    frame_width INTEGER,
    frame_height INTEGER

);
EOF

# Create the tags table
sqlite3 "$DB_PATH" <<EOF
CREATE TABLE tags (
    id INTEGER PRIMARY KEY,
    name TEXT UNIQUE COLLATE NOCASE

);
EOF

# Create the spritesheet_tags table
sqlite3 "$DB_PATH" <<EOF
CREATE TABLE spritesheet_tags (
    spritesheet_id INTEGER,
    tag_id INTEGER,
    PRIMARY KEY (spritesheet_id, tag_id),
    FOREIGN KEY (spritesheet_id) REFERENCES spritesheets(id),
    FOREIGN KEY (tag_id) REFERENCES tags(id)

);
EOF

