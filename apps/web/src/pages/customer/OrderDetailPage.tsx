import { useQuery } from '@tanstack/react-query';
import { useParams, Link } from 'react-router-dom';
import { ordersApi } from '@/services/api';

export default function OrderDetailPage() {
  const { orderId } = useParams<{ orderId: string }>();
  const { data: order, isLoading } = useQuery({
    queryKey: ['order', orderId],
    queryFn: () => ordersApi.getMyOrder(orderId!),
    enabled: !!orderId,
  });

  if (isLoading) return <p>Loading order...</p>;
  if (!order) return <p>Order not found. <Link to="/orders">Back to orders</Link></p>;

  return (
    <div style={{ padding: '1rem', maxWidth: 600 }}>
      <Link to="/orders">&larr; Back to Orders</Link>
      <h1>Order Details</h1>
      <p><strong>Status:</strong> {order.status}</p>
      <p><strong>Payment:</strong> {order.paymentStatus}</p>
      <p><strong>Date:</strong> {new Date(order.createdAt).toLocaleString()}</p>
      <h2>Items</h2>
      <table style={{ width: '100%', borderCollapse: 'collapse' }}>
        <thead>
          <tr>
            <th style={{ textAlign: 'left' }}>Title</th>
            <th>Qty</th>
            <th>Price</th>
          </tr>
        </thead>
        <tbody>
          {order.items?.map((item) => (
            <tr key={item.id} style={{ borderTop: '1px solid #eee' }}>
              <td>{item.title}</td>
              <td style={{ textAlign: 'center' }}>{item.quantity}</td>
              <td style={{ textAlign: 'right' }}>${(item.unitPrice / 100).toFixed(2)}</td>
            </tr>
          ))}
        </tbody>
        <tfoot>
          <tr>
            <td colSpan={2} style={{ textAlign: 'right', fontWeight: 'bold', paddingTop: '0.5rem' }}>Total:</td>
            <td style={{ textAlign: 'right', fontWeight: 'bold' }}>${(order.total / 100).toFixed(2)}</td>
          </tr>
        </tfoot>
      </table>
    </div>
  );
}
