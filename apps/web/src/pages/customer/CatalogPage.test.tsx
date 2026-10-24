import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router-dom';
import CatalogPage from '@/pages/customer/CatalogPage';

vi.mock('@/services/api', () => ({
  catalogApi: {
    listBooks: vi.fn(),
    listCategories: vi.fn(),
  },
}));

import { catalogApi } from '@/services/api';
const mockCatalog = catalogApi as {
  listBooks: ReturnType<typeof vi.fn>;
  listCategories: ReturnType<typeof vi.fn>;
};

const sampleBooks = [
  { id: 'b1', tenantId: 't1', title: 'Rust in Action', author: 'Tim McNamara', price: 3999, stockQty: 5, createdAt: '', updatedAt: '' },
  { id: 'b2', tenantId: 't1', title: 'The Pragmatic Programmer', author: 'David Thomas', price: 4999, stockQty: 10, createdAt: '', updatedAt: '' },
];

function makeClient() {
  return new QueryClient({ defaultOptions: { queries: { retry: false } } });
}

function renderPage() {
  mockCatalog.listCategories.mockResolvedValue([
    { id: 'c1', tenantId: 't1', name: 'Technology', slug: 'tech', createdAt: '' },
  ]);
  mockCatalog.listBooks.mockResolvedValue({ data: sampleBooks, total: 2, page: 1, perPage: 20 });

  return render(
    <MemoryRouter>
      <QueryClientProvider client={makeClient()}>
        <CatalogPage />
      </QueryClientProvider>
    </MemoryRouter>
  );
}

describe('CatalogPage', () => {
  beforeEach(() => vi.clearAllMocks());

  it('renders book titles', async () => {
    renderPage();
    expect(await screen.findByText('Rust in Action')).toBeInTheDocument();
    expect(await screen.findByText('The Pragmatic Programmer')).toBeInTheDocument();
  });

  it('renders book prices correctly', async () => {
    renderPage();
    expect(await screen.findByText('$39.99')).toBeInTheDocument();
    expect(await screen.findByText('$49.99')).toBeInTheDocument();
  });

  it('renders category filter options', async () => {
    renderPage();
    expect(await screen.findByText('Technology')).toBeInTheDocument();
  });

  it('shows page navigation', async () => {
    renderPage();
    expect(await screen.findByText(/page 1/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /previous/i })).toBeDisabled();
  });

  it('calls listBooks with search query on search button click', async () => {
    renderPage();

    const input = await screen.findByPlaceholderText(/search/i);
    fireEvent.change(input, { target: { value: 'Rust' } });
    fireEvent.click(screen.getByRole('button', { name: /search/i }));

    await waitFor(() => {
      expect(mockCatalog.listBooks).toHaveBeenCalledWith(
        expect.objectContaining({ q: 'Rust', page: 1 })
      );
    });
  });
});
