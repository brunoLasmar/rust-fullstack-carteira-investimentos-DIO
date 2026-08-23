-- Add up migration script here
CREATE TABLE IF NOT EXISTS owned_assets (
    id BIGSERIAL PRIMARY KEY NOT NULL,
    user_id BIGSERIAL NOT NULL REFERENCES users(id),
    asset_id BIGSERIAL NOT NULL REFERENCES assets(id),
    bought_for DOUBLE PRECISION NOT NULL,
    quantity_owned DOUBLE PRECISION NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
)