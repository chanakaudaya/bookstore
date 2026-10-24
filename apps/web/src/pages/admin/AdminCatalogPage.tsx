import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { catalogApi, ordersApi } from '@/services/api';
import type { Book } from '@/types';

export default function AdminCatalogPage() {
  const qc = useQueryClient();
  const [page, setPage] = useState(1);
  const [editingBook, setEditingBook] = useState<Book | null>(null);

  const { data, isLoading } = useQuery({
    queryKey: ['admin-books', page],
    queryFn: () => catalogApi.listBooks({ page, perPage: 20 }),
    keepPreviousData: true,
  });

  return (
    <div style={{ padding: '1rem' }}>
      <h1>Admin — Catalog</h1>
      {isLoading && <p>Loading...</p>}
      <table style={{ width: '100%', borderCollapse: 'collapse' }}>
        <thead>
          <tr>
            <th style={{ textAlign: 'left' }}>Title</th>
            <th>Author</th>
            <th>Price</th>
            <th>Stock</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {data?.data.map((book) => (
            <tr key={book.id} style={{ borderTop: '1px solid #eee' }}>
              <td>{book.title}</td>
              <td>{book.author}</td>
              <td>${(book.price / 100).toFixed(2)}</td>
              <td>{book.stockQty}</td>
              <td>
                <button onClick={() => setEditingBook(book)}>Edit</button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      <div style={{ marginTop: '1rem', display: 'flex', gap: '0.5rem' }}>
        <button disabled={page === 1} onClick={() => setPage((p) => p - 1)}>Previous</button>
        <span>Page {page}</span>
        <button disabled={(data?.data.length ?? 0) < 20} onClick={() => setPage((p) => p + 1)}>Next</button>
      </div>
      {editingBook && (
        <p style={{ marginTop: '1rem', color: '#888' }}>
          (Edit form for "{editingBook.title}" — integrate PUT /catalog/books/:id endpoint when admin catalog write APIs are added)
        </p>
      )}
    </div>
  );
}
