import { useQuery, keepPreviousData, useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { catalogApi } from '@/services/api';
import type { Book } from '@/types';

interface BookFormData {
  title: string;
  author: string;
  isbn: string;
  description: string;
  price: string;
  stock_qty: string;
  cover_image_url: string;
  category_id: string;
}

const emptyForm: BookFormData = {
  title: '',
  author: '',
  isbn: '',
  description: '',
  price: '',
  stock_qty: '',
  cover_image_url: '',
  category_id: '',
};

function bookToForm(book: Book): BookFormData {
  return {
    title: book.title,
    author: book.author,
    isbn: book.isbn ?? '',
    description: book.description ?? '',
    price: String(book.price / 100),
    stock_qty: String(book.stockQty),
    cover_image_url: book.coverImageUrl ?? '',
    category_id: book.categoryId ?? '',
  };
}

export default function AdminCatalogPage() {
  const qc = useQueryClient();
  const [page, setPage] = useState(1);
  const [showForm, setShowForm] = useState(false);
  const [editingBook, setEditingBook] = useState<Book | null>(null);
  const [form, setForm] = useState<BookFormData>(emptyForm);

  const { data: categories } = useQuery({
    queryKey: ['categories'],
    queryFn: () => catalogApi.listCategories(),
  });

  const { data, isLoading } = useQuery({
    queryKey: ['admin-books', page],
    queryFn: () => catalogApi.listBooks({ page, perPage: 20 }),
    placeholderData: keepPreviousData,
  });

  const createBook = useMutation({
    mutationFn: (payload: Parameters<typeof catalogApi.createBook>[0]) =>
      catalogApi.createBook(payload),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['admin-books'] });
      resetForm();
    },
  });

  const updateBook = useMutation({
    mutationFn: ({ id, payload }: { id: string; payload: Parameters<typeof catalogApi.updateBook>[1] }) =>
      catalogApi.updateBook(id, payload),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['admin-books'] });
      resetForm();
    },
  });

  const deleteBook = useMutation({
    mutationFn: (id: string) => catalogApi.deleteBook(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['admin-books'] }),
  });

  const resetForm = () => {
    setShowForm(false);
    setEditingBook(null);
    setForm(emptyForm);
  };

  const openCreate = () => {
    setForm(emptyForm);
    setEditingBook(null);
    setShowForm(true);
  };

  const openEdit = (book: Book) => {
    setForm(bookToForm(book));
    setEditingBook(book);
    setShowForm(true);
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const payload = {
      title: form.title,
      author: form.author,
      isbn: form.isbn || undefined,
      description: form.description || undefined,
      price: Math.round(parseFloat(form.price) * 100),
      stock_qty: parseInt(form.stock_qty, 10),
      cover_image_url: form.cover_image_url || undefined,
      category_id: form.category_id || undefined,
    };

    if (editingBook) {
      updateBook.mutate({ id: editingBook.id, payload });
    } else {
      createBook.mutate(payload);
    }
  };

  const fieldStyle: React.CSSProperties = { width: '100%', padding: '0.4rem', marginBottom: '0.5rem', boxSizing: 'border-box' };

  return (
    <div style={{ padding: '1.5rem' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
        <h1 style={{ margin: 0 }}>Admin — Catalog</h1>
        <button onClick={openCreate} style={{ padding: '0.5rem 1rem' }}>+ New Book</button>
      </div>

      {/* Book form */}
      {showForm && (
        <form
          onSubmit={handleSubmit}
          style={{ border: '1px solid #ddd', borderRadius: 8, padding: '1.5rem', marginBottom: '1.5rem', background: '#fff', maxWidth: 600 }}
        >
          <h2 style={{ margin: '0 0 1rem' }}>{editingBook ? 'Edit Book' : 'New Book'}</h2>

          <label>Title *</label>
          <input required style={fieldStyle} value={form.title} onChange={(e) => setForm({ ...form, title: e.target.value })} />

          <label>Author *</label>
          <input required style={fieldStyle} value={form.author} onChange={(e) => setForm({ ...form, author: e.target.value })} />

          <label>Category</label>
          <select style={fieldStyle} value={form.category_id} onChange={(e) => setForm({ ...form, category_id: e.target.value })}>
            <option value="">— None —</option>
            {categories?.map((c) => <option key={c.id} value={c.id}>{c.name}</option>)}
          </select>

          <label>ISBN</label>
          <input style={fieldStyle} value={form.isbn} onChange={(e) => setForm({ ...form, isbn: e.target.value })} />

          <label>Price (USD) *</label>
          <input required type="number" min="0" step="0.01" style={fieldStyle} value={form.price} onChange={(e) => setForm({ ...form, price: e.target.value })} />

          <label>Stock Quantity *</label>
          <input required type="number" min="0" style={fieldStyle} value={form.stock_qty} onChange={(e) => setForm({ ...form, stock_qty: e.target.value })} />

          <label>Cover Image URL</label>
          <input style={fieldStyle} value={form.cover_image_url} onChange={(e) => setForm({ ...form, cover_image_url: e.target.value })} />

          <label>Description</label>
          <textarea style={{ ...fieldStyle, height: 80 }} value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })} />

          <div style={{ display: 'flex', gap: '0.75rem', marginTop: '0.5rem' }}>
            <button type="submit" disabled={createBook.isPending || updateBook.isPending}>
              {editingBook ? 'Save Changes' : 'Create Book'}
            </button>
            <button type="button" onClick={resetForm}>Cancel</button>
          </div>
          {(createBook.isError || updateBook.isError) && <p style={{ color: 'red' }}>Save failed. Please try again.</p>}
        </form>
      )}

      {isLoading && <p>Loading...</p>}

      {/* Book table */}
      <table style={{ width: '100%', borderCollapse: 'collapse', background: '#fff' }}>
        <thead style={{ background: '#f5f5f5' }}>
          <tr>
            <th style={{ textAlign: 'left', padding: '0.5rem' }}>Title</th>
            <th style={{ padding: '0.5rem' }}>Author</th>
            <th style={{ padding: '0.5rem' }}>Price</th>
            <th style={{ padding: '0.5rem' }}>Stock</th>
            <th style={{ padding: '0.5rem' }}>Actions</th>
          </tr>
        </thead>
        <tbody>
          {data?.data.map((book) => (
            <tr key={book.id} style={{ borderTop: '1px solid #eee' }}>
              <td style={{ padding: '0.5rem' }}>{book.title}</td>
              <td style={{ padding: '0.5rem', color: '#666' }}>{book.author}</td>
              <td style={{ padding: '0.5rem', textAlign: 'right' }}>${(book.price / 100).toFixed(2)}</td>
              <td style={{ padding: '0.5rem', textAlign: 'center', color: book.stockQty === 0 ? 'red' : 'inherit' }}>{book.stockQty}</td>
              <td style={{ padding: '0.5rem', display: 'flex', gap: '0.5rem' }}>
                <button onClick={() => openEdit(book)}>Edit</button>
                <button
                  onClick={() => { if (window.confirm('Delete this book?')) deleteBook.mutate(book.id); }}
                  style={{ color: 'red' }}
                >
                  Delete
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>

      <div style={{ marginTop: '1rem', display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
        <button disabled={page === 1} onClick={() => setPage((p) => p - 1)}>Previous</button>
        <span>Page {page} — {data?.total ?? 0} total books</span>
        <button disabled={(data?.data.length ?? 0) < 20} onClick={() => setPage((p) => p + 1)}>Next</button>
      </div>
    </div>
  );
}
