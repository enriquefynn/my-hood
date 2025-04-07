CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE TYPE association_role AS ENUM ('admin', 'treasurer', 'member');

-- Add migration script here
CREATE TABLE IF NOT EXISTS "User" (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    password_hash VARCHAR(250),
    name VARCHAR(250) NOT NULL,
    birthday DATE NOT NULL,
    address VARCHAR(250) NOT NULL,
    activity VARCHAR(250),
    email VARCHAR(250) UNIQUE,
    personal_phone VARCHAR(25),
    commercial_phone VARCHAR(25),
    uses_whatsapp BOOLEAN NOT NULL DEFAULT TRUE,
    identities VARCHAR(1024),
    profile_url VARCHAR(128),
    deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);


CREATE TABLE IF NOT EXISTS "Association" (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(250) NOT NULL,    
    neighborhood VARCHAR(250) NOT NULL,
    country CHAR(2) NOT NULL,
    state VARCHAR(32) NOT NULL,
    address VARCHAR(250) NOT NULL,
    identity VARCHAR(250),
    public BOOLEAN NOT NULL DEFAULT FALSE,
    deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS "AssociationRoles" (
    user_id UUID NOT NULL,
    association_id UUID NOT NULL,
    role association_role NOT NULL,
    pending BOOLEAN NOT NULL DEFAULT TRUE,
    start_date DATE,
    end_date DATE,
    FOREIGN KEY (user_id) REFERENCES "User"(id),
    FOREIGN KEY (association_id) REFERENCES "Association"(id),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Financial data
-- If `amount < 0` it's an expense, otherwise it's an income.
CREATE TABLE IF NOT EXISTS "Transaction" (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    association_id UUID NOT NULL REFERENCES "Association"(id),
    creator_id UUID NOT NULL REFERENCES "User"(id),
    details VARCHAR(1024) NOT NULL,
    amount DECIMAL(9, 2) NOT NULL,
    proof_url VARCHAR(250),
    -- Date for which this expense/income is related.
    reference_date DATE NOT NULL,
    deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS "Charge" (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    association_id UUID NOT NULL REFERENCES "Association"(id),
    creator_id UUID NOT NULL REFERENCES "User"(id),
    details VARCHAR(1024),
    amount DECIMAL(9, 2) NOT NULL,
    file_url VARCHAR(250),
    -- Date for which this expense/income is related.
    reference_date DATE NOT NULL,
    deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS "Field" (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    association_id UUID NOT NULL REFERENCES "Association"(id),
    name VARCHAR(250) NOT NULL,
    description VARCHAR(1024),
    reservation_rules VARCHAR(1024),
    -- Latitude and longitude of the field.
    latitude DECIMAL(9, 6) NOT NULL,
    longitude DECIMAL(9, 6) NOT NULL,
    deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS "FieldReservation" (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    field_id UUID NOT NULL REFERENCES "Field"(id),
    user_id UUID NOT NULL REFERENCES "User"(id),
    description VARCHAR(1024),
    start_date TIMESTAMP WITH TIME ZONE NOT NULL,
    end_date TIMESTAMP WITH TIME ZONE NOT NULL,
    deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
   NEW.updated_at = NOW();
   RETURN NEW;
END;
$$ language "plpgsql";

CREATE TRIGGER trigger_name_before_update
BEFORE UPDATE ON "User"
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trigger_name_before_update
BEFORE UPDATE ON "Association"
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trigger_name_before_update
BEFORE UPDATE ON "AssociationRoles"
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trigger_name_before_update
BEFORE UPDATE ON "Transaction"
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trigger_name_before_update
BEFORE UPDATE ON "Charge"
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trigger_name_before_update
BEFORE UPDATE ON "Field"
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trigger_name_before_update
BEFORE UPDATE ON "FieldReservation"
EXECUTE FUNCTION update_updated_at_column();

-- TODO: Suggestions, Votes.