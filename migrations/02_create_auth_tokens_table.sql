-- Create auth_tokens table for JWT token management
CREATE TABLE IF NOT EXISTS auth_tokens (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    refresh_token_hash VARCHAR(255),
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    revoked BOOLEAN DEFAULT false,
    device_info VARCHAR(255)
);

-- Create indexes for efficient token lookups
CREATE INDEX IF NOT EXISTS idx_tokens_user_id ON auth_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_tokens_expires_at ON auth_tokens(expires_at);
CREATE INDEX IF NOT EXISTS idx_tokens_revoked ON auth_tokens(revoked);
