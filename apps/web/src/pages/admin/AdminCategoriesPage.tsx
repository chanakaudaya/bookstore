import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { catalogApi } from '@/services/api';
import type { Category } from '@/types';

interface CategoryFormData {
  name: string;
  slug: string;
  description: string;
}

const emptyForm: CategoryFormData = { name: '', slug: '', description: '' };

function categoryToForm(c: Category): CategoryFormData {
  return { name: c.name, slug: c.slug, description: c.description ?? '' };
}

function nameToSlug(name: string) {
  return name.toLowerCase().replace(/\s+/g, '-').replace(/[^a-z0-9-]/g, '');
}

export default function AdminCategoriesPage() {
  const qc = useQueryClient();
  const [showForm, setShowForm] = useState(false);
  const [editing, setEditing] = useState<Category | null>(null);
  const [form, setForm] = useState<CategoryFormData>(emptyForm);

  const { data: categories, isLoading } = useQuery({
    queryKey: ['categories'],
    queryFn: () => catalogApi.listCategories(),
  });

  const createCategory = useMutation({
    mutationFn: catalogApi.createCategory,
    onSuccess: () => { qc.invalidateQueries({ queryKey: ['categories'] }); resetForm(); },
  });

  const updateCategory = useMutation({
    mutationFn: ({ id, payload }: { id: string; payload: { name?: string; description?: string } }) =>
      catalogApi.updateCategory(id, payload),
    onSuccess: () => { qc.invalidateQueries({ queryKey: ['categories'] }); resetForm(); },
  });

  const deleteCategory = useMutation({
    mutationFn: (id: string) => catalogApi.deleteCategory(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['categories'] }),
  });

  const resetForm = () => { setShowForm(false); setEditing(null); setForm(emptyForm); };

  const openCreate = () => { setForm(emptyForm); setEditing(null); setShowForm(true); };

  const openEdit = (c: Category) => { setForm(categoryToForm(c)); setEditing(c); setShowForm(true); };

  const handleNameChange = (name: string) => {
    setForm((f) => ({
      ...f,
      name,
      slug: editing ? f.slug : nameToSlug(name),
    }));
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (editing) {
      updateCategory.mutate({ id: editing.id, payload: { name: form.name, description: form.description || undefined } });
    } else {
      createCategory.mutate({ name: form.name, slug: form.slug, description: form.description || undefined });
    }
  };

  const fieldStyle: React.CSSProperties = { width: '100%', padding: '0.4rem', marginBottom: '0.5rem', boxSizing: 'border-box' };

  return (
    <div style={{ padding: '1.5rem' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
        <h1 style={{ margin: 0 }}>Admin — Categories</h1>
        <button onClick={openCreate} style={{ padding: '0.5rem 1rem' }}>+ New Category</button>
      </div>

      {showForm && (
        <form
          onSubmit={handleSubmit}
          style={{ border: '1px solid #ddd', borderRadius: 8, padding: '1.5rem', marginBottom: '1.5rem', background: '#fff', maxWidth: 480 }}
        >
          <h2 style={{ margin: '0 0 1rem' }}>{editing ? 'Edit Category' : 'New Category'}</h2>

          <label>Name *</label>
          <input required style={fieldStyle} value={form.name} onChange={(e) => handleNameChange(e.target.value)} />

          <label>Slug *{editing && <span style={{ color: '#999', fontSize: '0.8rem' }}> (cannot be changed)</span>}</label>
          <input
            required
            style={{ ...fieldStyle, background: editing ? '#f5f5f5' : undefined }}
            value={form.slug}
            readOnly={!!editing}
            onChange={(e) => setForm({ ...form, slug: e.target.value })}
          />

          <label>Description</label>
          <textarea style={{ ...fieldStyle, height: 72 }} value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })} />

          <div style={{ display: 'flex', gap: '0.75rem', marginTop: '0.5rem' }}>
            <button type="submit" disabled={createCategory.isPending || updateCategory.isPending}>
              {editing ? 'Save Changes' : 'Create Category'}
            </button>
            <button type="button" onClick={resetForm}>Cancel</button>
          </div>
          {(createCategory.isError || updateCategory.isError) && (
            <p style={{ color: 'red' }}>
              {createCategory.error?.message ?? updateCategory.error?.message ?? 'Save failed.'}
            </p>
          )}
        </form>
      )}

      {isLoading && <p>Loading...</p>}

      <table style={{ width: '100%', borderCollapse: 'collapse', background: '#fff', maxWidth: 700 }}>
        <thead style={{ background: '#f5f5f5' }}>
          <tr>
            <th style={{ textAlign: 'left', padding: '0.5rem' }}>Name</th>
            <th style={{ padding: '0.5rem', textAlign: 'left' }}>Slug</th>
            <th style={{ padding: '0.5rem', textAlign: 'left' }}>Description</th>
            <th style={{ padding: '0.5rem' }}>Actions</th>
          </tr>
        </thead>
        <tbody>
          {categories?.map((c) => (
            <tr key={c.id} style={{ borderTop: '1px solid #eee' }}>
              <td style={{ padding: '0.5rem', fontWeight: 500 }}>{c.name}</td>
              <td style={{ padding: '0.5rem', fontFamily: 'monospace', fontSize: '0.85rem', color: '#666' }}>{c.slug}</td>
              <td style={{ padding: '0.5rem', color: '#666', fontSize: '0.9rem' }}>{c.description ?? '—'}</td>
              <td style={{ padding: '0.5rem', display: 'flex', gap: '0.5rem' }}>
                <button onClick={() => openEdit(c)}>Edit</button>
                <button
                  style={{ color: 'red' }}
                  onClick={() => {
                    if (window.confirm(`Delete category "${c.name}"? Books in this category will become uncategorized.`)) {
                      deleteCategory.mutate(c.id);
                    }
                  }}
                  disabled={deleteCategory.isPending}
                >
                  Delete
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      {categories?.length === 0 && <p style={{ color: '#888' }}>No categories yet.</p>}
    </div>
  );
}
