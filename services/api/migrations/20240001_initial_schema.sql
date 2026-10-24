-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Tenants
CREATE TABLE tenants (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name        TEXT NOT NULL,
    slug        TEXT NOT NULL UNIQUE,
    is_active   BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- User profiles (one per Keycloak user per tenant)
CREATE TABLE user_profiles (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id           UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    keycloak_user_id    TEXT NOT NULL,
    email               TEXT NOT NULL,
    first_name          TEXT,
    last_name           TEXT,
    phone               TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, keycloak_user_id)
);

CREATE INDEX idx_user_profiles_tenant ON user_profiles(tenant_id);
CREATE INDEX idx_user_profiles_keycloak ON user_profiles(keycloak_user_id);

-- Book categories
CREATE TABLE categories (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id   UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    slug        TEXT NOT NULL,
    description TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, slug)
);

CREATE INDEX idx_categories_tenant ON categories(tenant_id);

-- Books
CREATE TABLE books (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id           UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    category_id         UUID REFERENCES categories(id) ON DELETE SET NULL,
    title               TEXT NOT NULL,
    author              TEXT NOT NULL,
    isbn                TEXT,
    description         TEXT,
    price               BIGINT NOT NULL CHECK (price >= 0),  -- stored in smallest currency unit (e.g. cents)
    stock_qty           INTEGER NOT NULL DEFAULT 0 CHECK (stock_qty >= 0),
    cover_image_url     TEXT,
    published_at        TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_books_tenant ON books(tenant_id);
CREATE INDEX idx_books_category ON books(category_id);
CREATE INDEX idx_books_title ON books USING gin(to_tsvector('english', title));

-- Shipping addresses
CREATE TABLE shipping_addresses (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id     UUID NOT NULL REFERENCES user_profiles(id) ON DELETE CASCADE,
    tenant_id   UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    label       TEXT,
    line1       TEXT NOT NULL,
    line2       TEXT,
    city        TEXT NOT NULL,
    state       TEXT,
    postal_code TEXT NOT NULL,
    country     TEXT NOT NULL DEFAULT 'US',
    is_default  BOOLEAN NOT NULL DEFAULT FALSE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_shipping_addresses_user ON shipping_addresses(user_id, tenant_id);

-- Payment methods (metadata only — no raw card data)
CREATE TABLE payment_methods (
    id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id          UUID NOT NULL REFERENCES user_profiles(id) ON DELETE CASCADE,
    tenant_id        UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    label            TEXT NOT NULL,
    method_type      TEXT NOT NULL,        -- e.g. 'card', 'paypal', 'mock'
    last_four        TEXT,
    token_reference  TEXT,                 -- opaque token from future gateway integration
    is_default       BOOLEAN NOT NULL DEFAULT FALSE,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_payment_methods_user ON payment_methods(user_id, tenant_id);

-- Carts
CREATE TABLE carts (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id     UUID NOT NULL REFERENCES user_profiles(id) ON DELETE CASCADE,
    tenant_id   UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, tenant_id)
);

CREATE INDEX idx_carts_user ON carts(user_id, tenant_id);

-- Cart items
CREATE TABLE cart_items (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    cart_id     UUID NOT NULL REFERENCES carts(id) ON DELETE CASCADE,
    book_id     UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    title       TEXT NOT NULL,
    unit_price  BIGINT NOT NULL,
    quantity    INTEGER NOT NULL CHECK (quantity > 0),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (cart_id, book_id)
);

CREATE INDEX idx_cart_items_cart ON cart_items(cart_id);

-- Orders
CREATE TABLE orders (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id           UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id             UUID NOT NULL REFERENCES user_profiles(id),
    idempotency_key     UUID NOT NULL,
    shipping_address_id UUID NOT NULL REFERENCES shipping_addresses(id),
    payment_method_id   UUID NOT NULL REFERENCES payment_methods(id),
    status              TEXT NOT NULL DEFAULT 'confirmed',  -- confirmed | processing | shipped | delivered | cancelled | payment_failed
    payment_status      TEXT NOT NULL DEFAULT 'approved',   -- approved | declined
    total               BIGINT NOT NULL CHECK (total >= 0),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (idempotency_key, tenant_id)
);

CREATE INDEX idx_orders_tenant ON orders(tenant_id);
CREATE INDEX idx_orders_user ON orders(user_id, tenant_id);
CREATE INDEX idx_orders_status ON orders(tenant_id, status);

-- Order items (snapshot of book data at time of purchase)
CREATE TABLE order_items (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_id    UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    book_id     UUID NOT NULL REFERENCES books(id),
    title       TEXT NOT NULL,
    unit_price  BIGINT NOT NULL,
    quantity    INTEGER NOT NULL CHECK (quantity > 0)
);

CREATE INDEX idx_order_items_order ON order_items(order_id);

-- Audit log for admin mutations
CREATE TABLE audit_events (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id   UUID NOT NULL REFERENCES tenants(id),
    actor_id    TEXT NOT NULL,              -- Keycloak user ID
    action      TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id   UUID,
    payload     JSONB,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_events_tenant ON audit_events(tenant_id, created_at DESC);
