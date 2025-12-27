-- Create contacts table for friend/contact management
CREATE TABLE IF NOT EXISTS contacts (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    contact_user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status VARCHAR(20) DEFAULT 'pending', -- pending, accepted, blocked
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, contact_user_id),
    CHECK (user_id != contact_user_id)
);

-- Create indexes for efficient contact lookups
CREATE INDEX idx_contacts_user ON contacts(user_id);
CREATE INDEX idx_contacts_status ON contacts(user_id, status);
CREATE INDEX idx_contacts_contact_user ON contacts(contact_user_id);
