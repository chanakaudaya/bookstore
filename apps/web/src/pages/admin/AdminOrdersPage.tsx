import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { ordersApi } from '@/services/api';

const ORDER_STATUSES = ['confirmed', 'processing', 'shipped', 'delivered', 'cancelled'];

export default function AdminOrdersPage() {
  const qc = useQueryClient();
  const [page, setPage] = useState(1);

  const { data, isLoading } = useQuery({
    queryKey: ['admin-orders', page],
    queryFn: () => ordersApi.adminListOrders({ page, perPage: 20 }),
    keepPreviousData: true,
  });

  const updateStatus = useMutation({
    mutationFn: ({ id, status }: { id: string; status: string }) =>
      ordersApi.adminUpdateStatus(id, status),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['admin-orders'] }),
  });

  if (isLoading) return <p>Loading orders...</p>;

  return (
    <div style={{ padding: '1rem' }}>
      <h1>Admin — Orders</h1>
      <table style={{ width: '100%', borderCollapse: 'collapse' }}>
        <thead>
          <tr>
            <th style={{ textAlign: 'left' }}>Order ID</th>
            <th>Date</th>
            <th>Status</th>
            <th>Payment</th>
            <th>Total</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {data?.data.map((order) => (
            <tr key={order.id} style={{ borderTop: '1px solid #eee' }}>
              <td style={{ fontFamily: 'monospace', fontSize: '0.8rem' }}>{order.id.slice(0, 8)}...</td>
              <td>{new Date(order.createdAt).toLocaleDateString()}</td>
              <td>{order.status}</td>
              <td>{order.paymentStatus}</td>
              <td>${(order.total / 100).toFixed(2)}</td>
              <td>
                <select
                  value={order.status}
                  onChange={(e) => updateStatus.mutate({ id: order.id, status: e.target.value })}
                >
                  {ORDER_STATUSES.map((s) => (
                    <option key={s} value={s}>{s}</option>
                  ))}
                </select>
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
    </div>
  );
}
