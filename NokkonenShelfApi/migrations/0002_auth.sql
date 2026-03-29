ALTER TABLE users ADD COLUMN avatar_url TEXT;

CREATE TABLE oauth_accounts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,         -- 'google' | 'github'
    provider_user_id TEXT NOT NULL,
    provider_email TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (provider, provider_user_id)
);

CREATE INDEX oauth_accounts_user_id_idx ON oauth_accounts(user_id);

-- tower-sessions session store
CREATE TABLE tower_sessions (
    id TEXT PRIMARY KEY,
    data BYTEA NOT NULL,
    expiry_date TIMESTAMPTZ NOT NULL
);
