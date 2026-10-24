-- ============================================================
-- Sample data seed: Categories + Books for tenant-demo-001
-- ============================================================

\set TENANT_ID '00000000-0000-0000-0000-000000000001'

-- ----------------------------------------------------------------
-- Additional categories
-- ----------------------------------------------------------------
INSERT INTO categories (id, tenant_id, name, slug, description) VALUES
  (uuid_generate_v4(), :'TENANT_ID', 'Biography & Memoir',    'biography',      'Life stories of remarkable people'),
  (uuid_generate_v4(), :'TENANT_ID', 'Business & Finance',    'business',       'Entrepreneurship, investing, and management'),
  (uuid_generate_v4(), :'TENANT_ID', 'Self-Help',             'self-help',      'Personal growth and productivity'),
  (uuid_generate_v4(), :'TENANT_ID', 'Mystery & Thriller',    'mystery',        'Suspense, crime, and detective fiction'),
  (uuid_generate_v4(), :'TENANT_ID', 'Science Fiction',       'sci-fi',         'Speculative futures and technology')
ON CONFLICT (tenant_id, slug) DO NOTHING;

-- ----------------------------------------------------------------
-- Capture category IDs into temp table for convenience
-- ----------------------------------------------------------------
CREATE TEMP TABLE cat AS
  SELECT id, slug FROM categories WHERE tenant_id = '00000000-0000-0000-0000-000000000001';

-- ----------------------------------------------------------------
-- Books — Fiction
-- ----------------------------------------------------------------
INSERT INTO books (id, tenant_id, category_id, title, author, isbn, description, price, stock_qty, cover_image_url, published_at) VALUES
(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='fiction'),
 'The Great Gatsby', 'F. Scott Fitzgerald', '978-0-7432-7356-5',
 'A story of the fabulously wealthy Jay Gatsby and his love for the beautiful Daisy Buchanan, set in the Roaring Twenties.',
 999, 50,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1490528560i/4671.jpg',
 '1925-04-10'),

(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='fiction'),
 'To Kill a Mockingbird', 'Harper Lee', '978-0-06-112008-4',
 'The unforgettable novel of a childhood in a sleepy Southern town and the crisis of conscience that rocked it.',
 1299, 40,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1553383690i/2657.jpg',
 '1960-07-11'),

(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='fiction'),
 '1984', 'George Orwell', '978-0-452-28423-4',
 'A dystopian novel set in a totalitarian society where Big Brother watches your every move.',
 1099, 60,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1532714506i/40961427.jpg',
 '1949-06-08'),

(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='fiction'),
 'Pride and Prejudice', 'Jane Austen', '978-0-14-143951-8',
 'The story of Elizabeth Bennet and Mr. Darcy — a witty exploration of manners, marriage, and morality.',
 899, 35,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1320399351i/1885.jpg',
 '1813-01-28'),

(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='fiction'),
 'The Alchemist', 'Paulo Coelho', '978-0-06-231500-7',
 'A philosophical novel about a young Andalusian shepherd who travels from Spain to Egypt in search of treasure.',
 1199, 55,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1654371463i/18144590.jpg',
 '1988-01-01');

-- ----------------------------------------------------------------
-- Books — Science Fiction
-- ----------------------------------------------------------------
INSERT INTO books (id, tenant_id, category_id, title, author, isbn, description, price, stock_qty, cover_image_url, published_at) VALUES
(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='sci-fi'),
 'Dune', 'Frank Herbert', '978-0-441-17271-9',
 'Set on the desert planet Arrakis, Dune is the story of the boy Paul Atreides — heir to a noble family tasked with ruling a treacherous world.',
 1599, 45,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1555447414i/44767458.jpg',
 '1965-08-01'),

(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='sci-fi'),
 'The Hitchhiker''s Guide to the Galaxy', 'Douglas Adams', '978-0-345-39180-3',
 'Seconds before Earth is demolished to make way for a hyperspace bypass, Arthur Dent is swept off the planet by his friend Ford Prefect.',
 1099, 30,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1559986152i/386162.jpg',
 '1979-10-12'),

(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='sci-fi'),
 'Foundation', 'Isaac Asimov', '978-0-553-29335-7',
 'The story of a group of scientists who seek to preserve knowledge as civilizations begin to fall.',
 1299, 38,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1417900846i/29579.jpg',
 '1951-01-01'),

(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='sci-fi'),
 'The Martian', 'Andy Weir', '978-0-553-41802-6',
 'An astronaut becomes stranded alone on Mars and must use his ingenuity to survive while NASA plans a rescue.',
 1399, 50,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1413706054i/18007564.jpg',
 '2011-09-27');

-- ----------------------------------------------------------------
-- Books — Science & Technology
-- ----------------------------------------------------------------
INSERT INTO books (id, tenant_id, category_id, title, author, isbn, description, price, stock_qty, cover_image_url, published_at) VALUES
(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='science-tech'),
 'A Brief History of Time', 'Stephen Hawking', '978-0-553-38016-3',
 'A landmark volume in science writing by one of the great minds of our time, explaining black holes, the Big Bang, and the nature of time.',
 1499, 42,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1333578746i/3869.jpg',
 '1988-04-01'),

(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='science-tech'),
 'The Pragmatic Programmer', 'David Thomas & Andrew Hunt', '978-0-13-595705-9',
 'Your journey to mastery. Cuts through the increasing specialization and technicality of modern software development.',
 3999, 25,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1401432508i/4099.jpg',
 '1999-10-30'),

(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='science-tech'),
 'Clean Code', 'Robert C. Martin', '978-0-13-235088-4',
 'A handbook of agile software craftsmanship that teaches you how to write readable, maintainable code.',
 3599, 20,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1436202607i/3735293.jpg',
 '2008-08-01'),

(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='science-tech'),
 'Sapiens: A Brief History of Humankind', 'Yuval Noah Harari', '978-0-06-231609-7',
 'From a renowned historian comes a groundbreaking narrative of humanity''s creation and evolution.',
 1799, 60,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1595674533i/23692271.jpg',
 '2011-01-01');

-- ----------------------------------------------------------------
-- Books — Business & Finance
-- ----------------------------------------------------------------
INSERT INTO books (id, tenant_id, category_id, title, author, isbn, description, price, stock_qty, cover_image_url, published_at) VALUES
(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='business'),
 'The Lean Startup', 'Eric Ries', '978-0-307-88791-7',
 'How today''s entrepreneurs use continuous innovation to create radically successful businesses.',
 1699, 35,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1333576876i/10127019.jpg',
 '2011-09-13'),

(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='business'),
 'Zero to One', 'Peter Thiel', '978-0-8041-3929-8',
 'Notes on startups, or how to build the future. Every moment in business happens only once.',
 1599, 28,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1414347376i/18050143.jpg',
 '2014-09-16'),

(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='business'),
 'Rich Dad Poor Dad', 'Robert T. Kiyosaki', '978-1-61268-116-2',
 'What the rich teach their kids about money that the poor and middle class do not.',
 1299, 70,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1388196218i/69571.jpg',
 '1997-04-01');

-- ----------------------------------------------------------------
-- Books — Mystery & Thriller
-- ----------------------------------------------------------------
INSERT INTO books (id, tenant_id, category_id, title, author, isbn, description, price, stock_qty, cover_image_url, published_at) VALUES
(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='mystery'),
 'The Girl with the Dragon Tattoo', 'Stieg Larsson', '978-0-307-47347-9',
 'A journalist and a hacker investigate a decades-old disappearance in a wealthy Swedish family.',
 1499, 32,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1684526976i/2429135.jpg',
 '2005-01-01'),

(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='mystery'),
 'Gone Girl', 'Gillian Flynn', '978-0-307-58836-4',
 'On a warm summer morning in North Carthage, Missouri, it is Nick and Amy Dunne''s fifth wedding anniversary.',
 1299, 45,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1554086139i/19288043.jpg',
 '2012-06-05'),

(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='mystery'),
 'And Then There Were None', 'Agatha Christie', '978-0-312-33087-3',
 'Ten strangers are lured to an isolated island and begin to be killed one by one.',
 1099, 55,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1638425885i/16299.jpg',
 '1939-11-06');

-- ----------------------------------------------------------------
-- Books — Biography & Memoir
-- ----------------------------------------------------------------
INSERT INTO books (id, tenant_id, category_id, title, author, isbn, description, price, stock_qty, cover_image_url, published_at) VALUES
(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='biography'),
 'Steve Jobs', 'Walter Isaacson', '978-1-4516-4853-9',
 'Based on more than forty interviews with Jobs conducted over two years, as well as interviews with his family and colleagues.',
 1999, 30,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1511288482i/11084145.jpg',
 '2011-10-24'),

(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='biography'),
 'Educated', 'Tara Westover', '978-0-399-59050-4',
 'A memoir about a young woman who, kept out of school, leaves her survivalist family and goes on to earn a PhD from Cambridge University.',
 1599, 40,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1506026635i/35133922.jpg',
 '2018-02-20'),

(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='biography'),
 'Long Walk to Freedom', 'Nelson Mandela', '978-0-316-54585-6',
 'The autobiography of Nelson Mandela — the South African leader who became a symbol of the fight against apartheid.',
 1799, 25,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1388200077i/318431.jpg',
 '1994-10-20');

-- ----------------------------------------------------------------
-- Books — History
-- ----------------------------------------------------------------
INSERT INTO books (id, tenant_id, category_id, title, author, isbn, description, price, stock_qty, cover_image_url, published_at) VALUES
(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='history'),
 'The Diary of a Young Girl', 'Anne Frank', '978-0-553-29698-4',
 'The journal of a Jewish girl in Nazi-occupied Amsterdam, one of the most moving human documents to emerge from World War II.',
 1199, 65,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1560866271i/48855.jpg',
 '1947-06-25'),

(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='history'),
 'Guns, Germs, and Steel', 'Jared Diamond', '978-0-393-31755-8',
 'A Pulitzer Prize–winning account of why some civilizations conquered others and not vice versa.',
 1699, 28,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1453215833i/1842.jpg',
 '1997-03-01');

-- ----------------------------------------------------------------
-- Books — Self-Help
-- ----------------------------------------------------------------
INSERT INTO books (id, tenant_id, category_id, title, author, isbn, description, price, stock_qty, cover_image_url, published_at) VALUES
(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='self-help'),
 'Atomic Habits', 'James Clear', '978-0-7352-1129-2',
 'Tiny changes, remarkable results. An easy and proven way to build good habits and break bad ones.',
 1799, 80,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1655988385i/40121378.jpg',
 '2018-10-16'),

(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='self-help'),
 'The 7 Habits of Highly Effective People', 'Stephen R. Covey', '978-0-7432-6951-3',
 'A principle-centered approach for solving personal and professional problems.',
 1499, 45,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1421842784i/36072.jpg',
 '1989-08-15'),

(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='self-help'),
 'Think and Grow Rich', 'Napoleon Hill', '978-1-58542-433-1',
 'The classic bestseller that has inspired millions — distilling the success philosophies of the 500 richest people of the early 20th century.',
 1099, 60,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1463241782i/30186948.jpg',
 '1937-03-26');

-- ----------------------------------------------------------------
-- Books — Children
-- ----------------------------------------------------------------
INSERT INTO books (id, tenant_id, category_id, title, author, isbn, description, price, stock_qty, cover_image_url, published_at) VALUES
(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='children'),
 'Harry Potter and the Philosopher''s Stone', 'J.K. Rowling', '978-0-7475-3269-9',
 'The first book in the Harry Potter series — when a boy learns on his 11th birthday that he is the orphaned son of two powerful wizards.',
 1499, 100,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1474154022i/3.jpg',
 '1997-06-26'),

(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='children'),
 'The Lion, the Witch and the Wardrobe', 'C.S. Lewis', '978-0-06-447104-7',
 'Four adventurous siblings step through a wardrobe door and into the land of Narnia — a land frozen in eternal winter.',
 1199, 75,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1589795999i/100915.jpg',
 '1950-10-16'),

(uuid_generate_v4(), '00000000-0000-0000-0000-000000000001',
 (SELECT id FROM cat WHERE slug='children'),
 'Charlotte''s Web', 'E.B. White', '978-0-06-440313-3',
 'The timeless story of a pig named Wilbur and his friendship with a grey spider named Charlotte.',
 899, 85,
 'https://images-na.ssl-images-amazon.com/images/S/compressed.photo.goodreads.com/books/1666765182i/24178.jpg',
 '1952-10-15');

-- ----------------------------------------------------------------
-- Summary
-- ----------------------------------------------------------------
SELECT 
  c.name AS category,
  COUNT(b.id) AS book_count,
  MIN(b.price) AS min_price_cents,
  MAX(b.price) AS max_price_cents
FROM categories c
LEFT JOIN books b ON b.category_id = c.id AND b.tenant_id = c.tenant_id
WHERE c.tenant_id = '00000000-0000-0000-0000-000000000001'
GROUP BY c.name
ORDER BY c.name;
