-- Database initialization script for civic legislation platform

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

CREATE INDEX idx_bills_bill_number ON bills(bill_number);
CREATE INDEX idx_bills_year ON bills(year);
CREATE INDEX idx_bill_chunks_bill_id ON bill_chunks(bill_id);
CREATE INDEX idx_bill_chunks_embedding_id ON bill_chunks(embedding_id);

