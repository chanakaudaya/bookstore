import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { Link } from 'react-router-dom';
import { catalogApi } from '@/services/api';
import type { BookListParams } from '@/services/api';

export default function CatalogPage() {
  const [params, setParams] = useState<BookListParams>({ page: 1, perPage: 20 });
  const [search, setSearch] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string | undefined>();

  const { data: categories } = useQuery({
    queryKey: ['categories'],
    queryFn: () => catalogApi.listCategories(),
  });

  const { data, isLoading, isError } = useQuery({
    queryKey: ['books', params],
    queryFn: () => catalogApi.listBooks(params),
    keepPreviousData: true,
  });

  const handleSearch = () => {
    setParams({ ...params, q: search || undefined, categoryId: selectedCategory, page: 1 });
  };

  if (isLoading) return <p>Loading books...</p>;
  if (isError) return <p>Failed to load books.</p>;

  return (
    <div style={{ padding: '1rem' }}>
      <h1>Bookstore</h1>

      {/* Filters */}
      <div style={{ display: 'flex', gap: '0.5rem', marginBottom: '1rem' }}>
        <input
          type="text"
          placeholder="Search by title or author..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          style={{ flex: 1, padding: '0.5rem' }}
        />
        <select
          value={selectedCategory ?? ''}
          onChange={(e) => setSelectedCategory(e.target.value || undefined)}
        >
          <option value="">All Categories</option>
          {categories?.map((c) => (
            <option key={c.id} value={c.id}>
              {c.name}
            </option>
          ))}
        </select>
        <button onClick={handleSearch}>Search</button>
      </div>

      {/* Book grid */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))', gap: '1rem' }}>
        {data?.data.map((book) => (
          <Link
            key={book.id}
            to={`/books/${book.id}`}
            style={{ textDecoration: 'none', color: 'inherit', border: '1px solid #ddd', borderRadius: 8, padding: '1rem' }}
          >
            {book.coverImageUrl && (
              <img src={book.coverImageUrl} alt={book.title} style={{ width: '100%', height: 200, objectFit: 'cover' }} />
            )}
            <h3 style={{ margin: '0.5rem 0 0.25rem' }}>{book.title}</h3>
            <p style={{ margin: 0, color: '#666' }}>{book.author}</p>
            <p style={{ margin: '0.5rem 0 0', fontWeight: 'bold' }}>
              ${(book.price / 100).toFixed(2)}
            </p>
          </Link>
        ))}
      </div>

      {/* Pagination */}
      <div style={{ marginTop: '1rem', display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
        <button
          disabled={params.page === 1}
          onClick={() => setParams({ ...params, page: (params.page ?? 1) - 1 })}
        >
          Previous
        </button>
        <span>Page {params.page} of {Math.ceil((data?.total ?? 0) / (data?.perPage ?? 20))}</span>
        <button
          disabled={(data?.data.length ?? 0) < (data?.perPage ?? 20)}
          onClick={() => setParams({ ...params, page: (params.page ?? 1) + 1 })}
        >
          Next
        </button>
      </div>
    </div>
  );
}
