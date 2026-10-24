import { useQuery } from '@tanstack/react-query';
import { Link } from 'react-router-dom';
import { ordersApi } from '@/services/api';

export default function OrdersPage() {
  const { data, isLoading } = useQuery({
    queryKey: ['my-orders'],
    queryFn: () => ordersApi.listMyOrders(),
  });

  if (isLoading) return <p>Loading orders...</p>;

  return (
    <div style={{ padding: '1rem', maxWidth: 700 }}>
      <h1>My Orders</h1>
      {data?.data.length === 0 && <p>No orders yet. <Link to="/">Start shopping!</Link></p>}
      <table style={{ width: '100%', borderCollapse: 'collapse' }}>
        <thead>
          <tr>
            <th style={{ textAlign: 'left' }}>Order ID</th>
            <th>Date</th>
            <th>Status</th>
            <th>Total</th>
            <th />
          </tr>
        </thead>
        <tbody>
          {data?.data.map((order) => (
            <tr key={order.id} style={{ borderTop: '1px solid #eee' }}>
              <td style={{ fontFamily: 'monospace', fontSize: '0.8rem' }}>{order.id.slice(0, 8)}...</td>
              <td>{new Date(order.createdAt).toLocaleDateString()}</td>
              <td>{order.status}</td>
              <td>${(order.total / 100).toFixed(2)}</td>
              <td><Link to={`/orders/${order.id}`}>View</Link></td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
