ALTER TABLE owned_assets
ADD CONSTRAINT owned_assets_quantity_owned_positive
CHECK (quantity_owned > 0);
