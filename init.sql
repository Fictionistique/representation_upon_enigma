-- Database initialization script for civic legislation platform

-- Bills table
CREATE TABLE IF NOT EXISTS bills (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    bill_number TEXT UNIQUE NOT NULL,
    year INTEGER NOT NULL,
    session TEXT,
    status TEXT,
    introduction_date DATE,
    pdf_url TEXT,
    extracted_text TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS bill_chunks (
    id UUID PRIMARY KEY,
    bill_id UUID NOT NULL REFERENCES bills(id) ON DELETE CASCADE,
    chunk_index INTEGER NOT NULL,
    chunk_type TEXT, -- 'clause', 'section', 'preamble', etc.
    chunk_identifier TEXT, -- e.g., 'Clause 5', 'Section 2(a)'
    content TEXT NOT NULL,
    embedding_id TEXT, -- Reference to vector in Qdrant
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(bill_id, chunk_index)
);

-- Constituencies table (India parliamentary constituencies)
CREATE TABLE IF NOT EXISTS constituencies (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    state TEXT NOT NULL,
    code TEXT UNIQUE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Pincodes to constituency mapping
CREATE TABLE IF NOT EXISTS pincode_constituencies (
    id SERIAL PRIMARY KEY,
    pincode VARCHAR(6) NOT NULL,
    constituency_id INTEGER NOT NULL REFERENCES constituencies(id) ON DELETE CASCADE,
    UNIQUE(pincode)
);

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    real_name TEXT,
    age INTEGER,
    gender VARCHAR(20),
    pincode VARCHAR(6),
    constituency_id INTEGER REFERENCES constituencies(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Sessions table for login management
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_token TEXT UNIQUE NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Posts/Reviews table
CREATE TABLE IF NOT EXISTS posts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    bill_id UUID NOT NULL REFERENCES bills(id) ON DELETE CASCADE,
    stance VARCHAR(20) NOT NULL CHECK (stance IN ('Support', 'Oppose', 'Critique')),
    content TEXT NOT NULL,
    -- Moderation status: 'approved', 'rejected', 'pending_review'
    moderation_status VARCHAR(20) NOT NULL DEFAULT 'pending_review',
    moderation_reason TEXT,
    upvotes INTEGER DEFAULT 0,
    downvotes INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Post votes tracking (to prevent multiple votes from same user)
CREATE TABLE IF NOT EXISTS post_votes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    vote_type VARCHAR(10) NOT NULL CHECK (vote_type IN ('upvote', 'downvote')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(post_id, user_id)
);

-- Rate limiting table
CREATE TABLE IF NOT EXISTS rate_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    action_type VARCHAR(50) NOT NULL, -- 'post_create', 'vote', etc.
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_bills_bill_number ON bills(bill_number);
CREATE INDEX IF NOT EXISTS idx_bills_year ON bills(year);
CREATE INDEX IF NOT EXISTS idx_bill_chunks_bill_id ON bill_chunks(bill_id);
CREATE INDEX IF NOT EXISTS idx_bill_chunks_embedding_id ON bill_chunks(embedding_id);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(session_token);
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_posts_bill_id ON posts(bill_id);
CREATE INDEX IF NOT EXISTS idx_posts_user_id ON posts(user_id);
CREATE INDEX IF NOT EXISTS idx_posts_moderation_status ON posts(moderation_status);
CREATE INDEX IF NOT EXISTS idx_rate_limits_user_action ON rate_limits(user_id, action_type);
CREATE INDEX IF NOT EXISTS idx_rate_limits_timestamp ON rate_limits(timestamp);
CREATE INDEX IF NOT EXISTS idx_pincode_constituencies_pincode ON pincode_constituencies(pincode);

-- Insert sample constituencies (major Indian cities/areas)
INSERT INTO constituencies (name, state, code) VALUES 
    ('Mumbai South', 'Maharashtra', 'MH-MS'),
    ('Mumbai North', 'Maharashtra', 'MH-MN'),
    ('Mumbai North Central', 'Maharashtra', 'MH-MNC'),
    ('Delhi Central', 'Delhi', 'DL-C'),
    ('Delhi East', 'Delhi', 'DL-E'),
    ('Delhi North', 'Delhi', 'DL-N'),
    ('Delhi South', 'Delhi', 'DL-S'),
    ('Delhi West', 'Delhi', 'DL-W'),
    ('Bangalore North', 'Karnataka', 'KA-BN'),
    ('Bangalore South', 'Karnataka', 'KA-BS'),
    ('Bangalore Central', 'Karnataka', 'KA-BC'),
    ('Chennai North', 'Tamil Nadu', 'TN-CN'),
    ('Chennai South', 'Tamil Nadu', 'TN-CS'),
    ('Chennai Central', 'Tamil Nadu', 'TN-CC'),
    ('Kolkata North', 'West Bengal', 'WB-KN'),
    ('Kolkata South', 'West Bengal', 'WB-KS'),
    ('Kolkata Dakshin', 'West Bengal', 'WB-KD'),
    ('Hyderabad', 'Telangana', 'TS-HY'),
    ('Secunderabad', 'Telangana', 'TS-SC'),
    ('Ahmedabad East', 'Gujarat', 'GJ-AE'),
    ('Ahmedabad West', 'Gujarat', 'GJ-AW'),
    ('Pune', 'Maharashtra', 'MH-PU'),
    ('Jaipur', 'Rajasthan', 'RJ-JP'),
    ('Lucknow', 'Uttar Pradesh', 'UP-LK'),
    ('Varanasi', 'Uttar Pradesh', 'UP-VN')
ON CONFLICT (code) DO NOTHING;

-- Insert sample pincode mappings
INSERT INTO pincode_constituencies (pincode, constituency_id) VALUES 
    ('400001', (SELECT id FROM constituencies WHERE code = 'MH-MS')),
    ('400002', (SELECT id FROM constituencies WHERE code = 'MH-MS')),
    ('400003', (SELECT id FROM constituencies WHERE code = 'MH-MS')),
    ('400050', (SELECT id FROM constituencies WHERE code = 'MH-MN')),
    ('400051', (SELECT id FROM constituencies WHERE code = 'MH-MN')),
    ('110001', (SELECT id FROM constituencies WHERE code = 'DL-C')),
    ('110002', (SELECT id FROM constituencies WHERE code = 'DL-C')),
    ('110003', (SELECT id FROM constituencies WHERE code = 'DL-N')),
    ('560001', (SELECT id FROM constituencies WHERE code = 'KA-BN')),
    ('560002', (SELECT id FROM constituencies WHERE code = 'KA-BS')),
    ('600001', (SELECT id FROM constituencies WHERE code = 'TN-CN')),
    ('600002', (SELECT id FROM constituencies WHERE code = 'TN-CS')),
    ('700001', (SELECT id FROM constituencies WHERE code = 'WB-KN')),
    ('700002', (SELECT id FROM constituencies WHERE code = 'WB-KS')),
    ('500001', (SELECT id FROM constituencies WHERE code = 'TS-HY')),
    ('380001', (SELECT id FROM constituencies WHERE code = 'GJ-AE')),
    ('411001', (SELECT id FROM constituencies WHERE code = 'MH-PU')),
    ('302001', (SELECT id FROM constituencies WHERE code = 'RJ-JP')),
    ('226001', (SELECT id FROM constituencies WHERE code = 'UP-LK')),
    ('221001', (SELECT id FROM constituencies WHERE code = 'UP-VN'))
ON CONFLICT (pincode) DO NOTHING;
