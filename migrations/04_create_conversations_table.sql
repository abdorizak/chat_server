-- Create conversations table for 1-to-1 chats
CREATE TABLE IF NOT EXISTS conversations (
    id SERIAL PRIMARY KEY,
    participant_1 INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    participant_2 INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(participant_1, participant_2),
    CHECK (participant_1 < participant_2)
);

-- Create indexes for efficient conversation lookups
CREATE INDEX IF NOT EXISTS idx_conv_participants ON conversations(participant_1, participant_2);
CREATE INDEX IF NOT EXISTS idx_conv_participant_1 ON conversations(participant_1);
CREATE INDEX IF NOT EXISTS idx_conv_participant_2 ON conversations(participant_2);
