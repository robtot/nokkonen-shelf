CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE bookcases (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    position INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE shelves (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    bookcase_id UUID NOT NULL REFERENCES bookcases(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    position INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE books (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    shelf_id UUID NOT NULL REFERENCES shelves(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    color TEXT NOT NULL DEFAULT '#8B4513',
    position INTEGER NOT NULL,
    open_library_url TEXT,
    cover_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX bookcases_user_id_idx ON bookcases(user_id);
CREATE INDEX shelves_bookcase_id_idx ON shelves(bookcase_id);
CREATE INDEX books_shelf_id_idx ON books(shelf_id);
