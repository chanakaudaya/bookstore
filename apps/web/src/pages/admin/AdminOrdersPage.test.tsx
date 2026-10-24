import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router-dom';
import AdminOrdersPage from '@/pages/admin/AdminOrdersPage';

vi.mock('@/services/api', () => ({
  ordersApi: {
    adminListOrders: vi.fn(),
    adminUpdateStatus: vi.fn(),
  },
}));

import { ordersApi } from '@/services/api';
const mockOrdersApi = ordersApi as {
  adminListOrders: ReturnType<typeof vi.fn>;
  adminUpdateStatus: ReturnType<typeof vi.fn>;
};

const sampleOrders = [
  {
    id: 'ord-001',
    tenantId: 't1',
    userId: 'u1',
    status: 'confirmed',
    paymentStatus: 'approved',
    total: 9999,
    shippingAddressId: 'addr1',
    paymentMethodId: 'pm1',
    createdAt: '2024-01-15T10:00:00Z',
    updatedAt: '2024-01-15T10:00:00Z',
  },
];

function makeClient() {
  return new QueryClient({ defaultOptions: { queries: { retry: false } } });
}

function renderPage() {
  mockOrdersApi.adminListOrders.mockResolvedValue({ data: sampleOrders, total: 1, page: 1, perPage: 20 });
  mockOrdersApi.adminUpdateStatus.mockResolvedValue({ ...sampleOrders[0], status: 'processing' });

  return render(
    <MemoryRouter>
      <QueryClientProvider client={makeClient()}>
        <AdminOrdersPage />
      </QueryClientProvider>
    </MemoryRouter>
  );
}

describe('AdminOrdersPage', () => {
  beforeEach(() => vi.clearAllMocks());

  it('renders order rows', async () => {
    renderPage();
    expect(await screen.findByText('confirmed')).toBeInTheDocument();
    expect(await screen.findByText('approved')).toBeInTheDocument();
    expect(await screen.findByText('$99.99')).toBeInTheDocument();
  });

  it('calls adminUpdateStatus when status dropdown changes', async () => {
    renderPage();

    const select = await screen.findByRole('combobox');
    fireEvent.change(select, { target: { value: 'processing' } });

    await waitFor(() => {
      expect(mockOrdersApi.adminUpdateStatus).toHaveBeenCalledWith('ord-001', 'processing');
    });
  });

  it('shows pagination controls', async () => {
    renderPage();
    expect(await screen.findByRole('button', { name: /previous/i })).toBeInTheDocument();
    expect(await screen.findByRole('button', { name: /next/i })).toBeInTheDocument();
  });
});
