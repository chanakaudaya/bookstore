-- Seed demo tenant
INSERT INTO tenants (id, name, slug)
VALUES ('00000000-0000-0000-0000-000000000001', 'Demo Bookstore', 'tenant-demo-001')
ON CONFLICT DO NOTHING;

-- Seed demo categories
INSERT INTO categories (id, tenant_id, name, slug, description)
VALUES
    (uuid_generate_v4(), '00000000-0000-0000-0000-000000000001', 'Fiction', 'fiction', 'Novels and short stories'),
    (uuid_generate_v4(), '00000000-0000-0000-0000-000000000001', 'Science & Technology', 'science-tech', 'Science and technology books'),
    (uuid_generate_v4(), '00000000-0000-0000-0000-000000000001', 'History', 'history', 'Historical accounts and biographies'),
    (uuid_generate_v4(), '00000000-0000-0000-0000-000000000001', 'Children', 'children', 'Books for young readers')
ON CONFLICT DO NOTHING;
