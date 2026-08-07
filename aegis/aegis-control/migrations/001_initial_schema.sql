CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE campaigns (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    target_id UUID NOT NULL,
    status VARCHAR(50) DEFAULT 'pending',
    total_execs BIGINT DEFAULT 0,
    execs_per_sec FLOAT DEFAULT 0,
    total_crashes INT DEFAULT 0,
    unique_crashes INT DEFAULT 0,
    corpus_size INT DEFAULT 0,
    coverage_percent FLOAT DEFAULT 0,
    start_time TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE targets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    type VARCHAR(50) NOT NULL,
    command TEXT NOT NULL,
    args TEXT[],
    input_mode VARCHAR(50) DEFAULT 'stdin',
    timeout_seconds INT DEFAULT 5,
    memory_limit_mb INT DEFAULT 256,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE workers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    hostname VARCHAR(255) NOT NULL,
    address VARCHAR(255) NOT NULL,
    status VARCHAR(50) DEFAULT 'online',
    last_heartbeat TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    registered_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE crashes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    campaign_id UUID REFERENCES campaigns(id) ON DELETE CASCADE,
    crash_type VARCHAR(100) NOT NULL,
    stack_hash BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) DEFAULT 'engineer',
    api_key VARCHAR(255) UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
