import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router-dom';
import CartPage from '@/pages/customer/CartPage';

vi.mock('@/services/api', () => ({
  cartApi: {
    getCart: vi.fn(),
    upsertItem: vi.fn(),
    clearCart: vi.fn(),
  },
}));

import { cartApi } from '@/services/api';
const mockCartApi = cartApi as { getCart: ReturnType<typeof vi.fn>; upsertItem: ReturnType<typeof vi.fn>; clearCart: ReturnType<typeof vi.fn> };

function makeQueryClient() {
  return new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
}

function renderCartPage(cartData: object) {
  mockCartApi.getCart.mockResolvedValue(cartData);
  return render(
    <MemoryRouter>
      <QueryClientProvider client={makeQueryClient()}>
        <CartPage />
      </QueryClientProvider>
    </MemoryRouter>
  );
}

describe('CartPage', () => {
  beforeEach(() => vi.clearAllMocks());

  it('shows empty cart message when cart has no items', async () => {
    renderCartPage({ id: 'c1', userId: 'u1', tenantId: 't1', items: [], subtotal: 0, createdAt: '', updatedAt: '' });
    expect(await screen.findByText(/your cart is empty/i)).toBeInTheDocument();
  });

  it('renders cart items with correct totals', async () => {
    renderCartPage({
      id: 'c1',
      userId: 'u1',
      tenantId: 't1',
      subtotal: 3998,
      createdAt: '',
      updatedAt: '',
      items: [
        { id: 'i1', cartId: 'c1', bookId: 'b1', title: 'Rust Programming', unitPrice: 1999, quantity: 2, createdAt: '' },
      ],
    });

    expect(await screen.findByText('Rust Programming')).toBeInTheDocument();
    // Unit price
    expect(await screen.findByText('$19.99')).toBeInTheDocument();
    // Line total
    expect(await screen.findByText('$39.98')).toBeInTheDocument();
  });

  it('shows subtotal', async () => {
    renderCartPage({
      id: 'c1', userId: 'u1', tenantId: 't1', subtotal: 999, createdAt: '', updatedAt: '',
      items: [{ id: 'i1', cartId: 'c1', bookId: 'b1', title: 'Test Book', unitPrice: 999, quantity: 1, createdAt: '' }],
    });

    const subtotals = await screen.findAllByText('$9.99');
    expect(subtotals.length).toBeGreaterThanOrEqual(1);
  });
});
