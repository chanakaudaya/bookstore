import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Link } from 'react-router-dom';
import { cartApi } from '@/services/api';

export default function CartPage() {
  const qc = useQueryClient();

  const { data: cart, isLoading } = useQuery({
    queryKey: ['cart'],
    queryFn: () => cartApi.getCart(),
  });

  const removeItem = useMutation({
    mutationFn: (bookId: string) => cartApi.upsertItem({ bookId, quantity: 0 }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['cart'] }),
  });

  const clearCart = useMutation({
    mutationFn: () => cartApi.clearCart(),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['cart'] }),
  });

  if (isLoading) return <p>Loading cart...</p>;

  const items = cart?.items ?? [];

  return (
    <div style={{ padding: '1rem', maxWidth: 700 }}>
      <h1>Your Cart</h1>

      {items.length === 0 ? (
        <div>
          <p>Your cart is empty.</p>
          <Link to="/">Continue Shopping</Link>
        </div>
      ) : (
        <>
          <table style={{ width: '100%', borderCollapse: 'collapse' }}>
            <thead>
              <tr>
                <th style={{ textAlign: 'left' }}>Title</th>
                <th>Qty</th>
                <th>Price</th>
                <th>Total</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {items.map((item) => (
                <tr key={item.id} style={{ borderTop: '1px solid #eee' }}>
                  <td>{item.title}</td>
                  <td style={{ textAlign: 'center' }}>{item.quantity}</td>
                  <td style={{ textAlign: 'right' }}>${(item.unitPrice / 100).toFixed(2)}</td>
                  <td style={{ textAlign: 'right' }}>${((item.unitPrice * item.quantity) / 100).toFixed(2)}</td>
                  <td>
                    <button onClick={() => removeItem.mutate(item.bookId)}>Remove</button>
                  </td>
                </tr>
              ))}
            </tbody>
            <tfoot>
              <tr>
                <td colSpan={3} style={{ textAlign: 'right', fontWeight: 'bold', paddingTop: '0.5rem' }}>Subtotal:</td>
                <td style={{ textAlign: 'right', fontWeight: 'bold' }}>${((cart?.subtotal ?? 0) / 100).toFixed(2)}</td>
                <td />
              </tr>
            </tfoot>
          </table>

          <div style={{ marginTop: '1rem', display: 'flex', gap: '1rem' }}>
            <button onClick={() => clearCart.mutate()} disabled={clearCart.isPending}>
              Clear Cart
            </button>
            <Link to="/checkout">
              <button style={{ padding: '0.75rem 2rem' }}>Proceed to Checkout</button>
            </Link>
          </div>
        </>
      )}
    </div>
  );
}
