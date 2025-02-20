CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
-- Add migration script here
CREATE TABLE IF NOT EXISTS "User" (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(250) NOT NULL,
    birthday DATE NOT NULL,
    address VARCHAR(250) NOT NULL,
    activity VARCHAR(250),
    email VARCHAR(250) UNIQUE,
    personal_phone VARCHAR(25),
    commercial_phone VARCHAR(25),
    uses_whatsapp BOOLEAN NOT NULL,
    identities VARCHAR(1024),
    profile_url VARCHAR(128),
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
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS "UserAssociation" (
    user_id UUID NOT NULL REFERENCES "User"(id),
    association_id UUID NOT NULL REFERENCES "Association"(id),
    PRIMARY KEY (user_id, association_id),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS "AssociationAdmin" (
    user_id UUID NOT NULL,
    association_id UUID NOT NULL,
    PRIMARY KEY (user_id, association_id),
    FOREIGN KEY (user_id) REFERENCES "User"(id),
    FOREIGN KEY (association_id) REFERENCES "Association"(id),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS "AssociationTreasurer" (
    user_id UUID NOT NULL,
    association_id UUID NOT NULL,
    PRIMARY KEY (user_id, association_id),
    FOREIGN KEY (user_id) REFERENCES "User"(id),
    FOREIGN KEY (association_id) REFERENCES "Association"(id),
    start_date DATE NOT NULL,
    end_date DATE,
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
    -- Date for which this expense/income is related.
    reference_date DATE NOT NULL,
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
$$ language 'plpgsql';

CREATE TRIGGER trigger_name_before_update
BEFORE UPDATE ON "User"
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trigger_name_before_update
BEFORE UPDATE ON "Association"
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trigger_name_before_update
BEFORE UPDATE ON "UserAssociation"
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trigger_name_before_update
BEFORE UPDATE ON "AssociationAdmin"
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trigger_name_before_update
BEFORE UPDATE ON "AssociationTreasurer"
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trigger_name_before_update
BEFORE UPDATE ON "Transaction"
EXECUTE FUNCTION update_updated_at_column();

-- TODO: Suggestions, Votes.