-- CCO Metrics Backend - Seed Pricing Data
-- Version: 2025.11.2
-- Purpose: Insert default pricing for Claude models (January 2025 pricing)

-- Claude Opus 4 (Highest tier - strategic reasoning)
INSERT OR REPLACE INTO model_tiers (
    model,
    tier,
    provider,
    input_cost_per_1m,
    output_cost_per_1m,
    cache_write_cost_per_1m,
    cache_read_cost_per_1m,
    active
) VALUES (
    'claude-opus-4-1-20250805',
    'opus',
    'anthropic',
    15.00,    -- $15 per 1M input tokens
    75.00,    -- $75 per 1M output tokens
    18.75,    -- $18.75 per 1M cache write tokens (125% of input)
    1.50,     -- $1.50 per 1M cache read tokens (10% of input)
    1
);

-- Legacy Opus naming (for compatibility)
INSERT OR REPLACE INTO model_tiers (
    model,
    tier,
    provider,
    input_cost_per_1m,
    output_cost_per_1m,
    cache_write_cost_per_1m,
    cache_read_cost_per_1m,
    active
) VALUES (
    'claude-opus-4',
    'opus',
    'anthropic',
    15.00,
    75.00,
    18.75,
    1.50,
    1
);

-- Claude Sonnet 4.5 (Mid tier - intelligent coding)
INSERT OR REPLACE INTO model_tiers (
    model,
    tier,
    provider,
    input_cost_per_1m,
    output_cost_per_1m,
    cache_write_cost_per_1m,
    cache_read_cost_per_1m,
    active
) VALUES (
    'claude-sonnet-4-5-20250929',
    'sonnet',
    'anthropic',
    3.00,     -- $3 per 1M input tokens
    15.00,    -- $15 per 1M output tokens
    3.75,     -- $3.75 per 1M cache write tokens (125% of input)
    0.30,     -- $0.30 per 1M cache read tokens (10% of input)
    1
);

-- Legacy Sonnet naming (for compatibility)
INSERT OR REPLACE INTO model_tiers (
    model,
    tier,
    provider,
    input_cost_per_1m,
    output_cost_per_1m,
    cache_write_cost_per_1m,
    cache_read_cost_per_1m,
    active
) VALUES (
    'claude-sonnet-4.5',
    'sonnet',
    'anthropic',
    3.00,
    15.00,
    3.75,
    0.30,
    1
);

INSERT OR REPLACE INTO model_tiers (
    model,
    tier,
    provider,
    input_cost_per_1m,
    output_cost_per_1m,
    cache_write_cost_per_1m,
    cache_read_cost_per_1m,
    active
) VALUES (
    'claude-sonnet-4',
    'sonnet',
    'anthropic',
    3.00,
    15.00,
    3.75,
    0.30,
    1
);

-- Claude Sonnet 3.5 (Previous generation)
INSERT OR REPLACE INTO model_tiers (
    model,
    tier,
    provider,
    input_cost_per_1m,
    output_cost_per_1m,
    cache_write_cost_per_1m,
    cache_read_cost_per_1m,
    active
) VALUES (
    'claude-sonnet-3.5',
    'sonnet',
    'anthropic',
    3.00,
    15.00,
    3.75,
    0.30,
    1
);

-- Claude Haiku 4.5 (Low tier - fast, cost-effective)
INSERT OR REPLACE INTO model_tiers (
    model,
    tier,
    provider,
    input_cost_per_1m,
    output_cost_per_1m,
    cache_write_cost_per_1m,
    cache_read_cost_per_1m,
    active
) VALUES (
    'claude-haiku-4-5-20251001',
    'haiku',
    'anthropic',
    0.80,     -- $0.80 per 1M input tokens
    4.00,     -- $4.00 per 1M output tokens
    1.00,     -- $1.00 per 1M cache write tokens (125% of input)
    0.08,     -- $0.08 per 1M cache read tokens (10% of input)
    1
);

-- Legacy Haiku naming (for compatibility)
INSERT OR REPLACE INTO model_tiers (
    model,
    tier,
    provider,
    input_cost_per_1m,
    output_cost_per_1m,
    cache_write_cost_per_1m,
    cache_read_cost_per_1m,
    active
) VALUES (
    'claude-haiku-4.5',
    'haiku',
    'anthropic',
    0.80,
    4.00,
    1.00,
    0.08,
    1
);

-- OpenAI GPT-4 (for comparison)
INSERT OR REPLACE INTO model_tiers (
    model,
    tier,
    provider,
    input_cost_per_1m,
    output_cost_per_1m,
    cache_write_cost_per_1m,
    cache_read_cost_per_1m,
    active
) VALUES (
    'gpt-4',
    'other',
    'openai',
    30.00,    -- $30 per 1M input tokens
    60.00,    -- $60 per 1M output tokens
    0.0,      -- No cache pricing for OpenAI
    0.0,
    1
);

-- OpenAI GPT-4 Turbo
INSERT OR REPLACE INTO model_tiers (
    model,
    tier,
    provider,
    input_cost_per_1m,
    output_cost_per_1m,
    cache_write_cost_per_1m,
    cache_read_cost_per_1m,
    active
) VALUES (
    'gpt-4-turbo',
    'other',
    'openai',
    10.00,    -- $10 per 1M input tokens
    30.00,    -- $30 per 1M output tokens
    0.0,
    0.0,
    1
);

-- OpenAI GPT-3.5 Turbo
INSERT OR REPLACE INTO model_tiers (
    model,
    tier,
    provider,
    input_cost_per_1m,
    output_cost_per_1m,
    cache_write_cost_per_1m,
    cache_read_cost_per_1m,
    active
) VALUES (
    'gpt-3.5-turbo',
    'other',
    'openai',
    0.50,     -- $0.50 per 1M input tokens
    1.50,     -- $1.50 per 1M output tokens
    0.0,
    0.0,
    1
);

-- Ollama models (self-hosted, free)
INSERT OR REPLACE INTO model_tiers (
    model,
    tier,
    provider,
    input_cost_per_1m,
    output_cost_per_1m,
    cache_write_cost_per_1m,
    cache_read_cost_per_1m,
    active
) VALUES (
    'ollama/llama3-70b',
    'other',
    'ollama',
    0.0,      -- Free (self-hosted)
    0.0,
    0.0,
    0.0,
    1
);

INSERT OR REPLACE INTO model_tiers (
    model,
    tier,
    provider,
    input_cost_per_1m,
    output_cost_per_1m,
    cache_write_cost_per_1m,
    cache_read_cost_per_1m,
    active
) VALUES (
    'ollama/mistral',
    'other',
    'ollama',
    0.0,
    0.0,
    0.0,
    0.0,
    1
);

-- Insert pricing summary into config
INSERT OR REPLACE INTO config (key, value, description) VALUES (
    'pricing_version',
    '2025.01',
    'Pricing data version (January 2025)'
);

-- Create view for quick pricing lookup
CREATE VIEW IF NOT EXISTS v_active_pricing AS
SELECT
    model,
    tier,
    provider,
    input_cost_per_1m,
    output_cost_per_1m,
    cache_write_cost_per_1m,
    cache_read_cost_per_1m,
    ROUND((input_cost_per_1m + output_cost_per_1m) / 2.0, 2) as avg_cost_per_1m
FROM model_tiers
WHERE active = 1
ORDER BY tier, avg_cost_per_1m DESC;
