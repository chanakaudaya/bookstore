import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Link } from 'react-router-dom';
import { cartApi } from '@/services/api';

export default function CartPage() {
  const qc = useQueryClient();

  const { data: cart, isLoading } = useQuery({
    queryKey: ['cart'],
    queryFn: () => cartApi.getCart(),
  });

  const upsertItem = useMutation({
    mutationFn: ({ bookId, quantity }: { bookId: string; quantity: number }) =>
      cartApi.upsertItem({ bookId, quantity }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['cart'] }),
  });

  const clearCart = useMutation({
    mutationFn: () => cartApi.clearCart(),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['cart'] }),
  });

  if (isLoading) return <p style={{ padding: '1rem' }}>Loading cart...</p>;

  const items = cart?.items ?? [];

  return (
    <div style={{ padding: '1.5rem', maxWidth: 760 }}>
      <h1>Your Cart</h1>

      {items.length === 0 ? (
        <div>
          <p>Your cart is empty.</p>
          <Link to="/">Continue Shopping</Link>
        </div>
      ) : (
        <>
          <table style={{ width: '100%', borderCollapse: 'collapse', background: '#fff' }}>
            <thead style={{ background: '#f5f5f5' }}>
              <tr>
                <th style={{ textAlign: 'left', padding: '0.5rem' }}>Title</th>
                <th style={{ padding: '0.5rem' }}>Unit Price</th>
                <th style={{ padding: '0.5rem' }}>Quantity</th>
                <th style={{ padding: '0.5rem' }}>Line Total</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {items.map((item) => (
                <tr key={item.id} style={{ borderTop: '1px solid #eee' }}>
                  <td style={{ padding: '0.5rem' }}>{item.title}</td>
                  <td style={{ padding: '0.5rem', textAlign: 'right' }}>
                    ${(item.unitPrice / 100).toFixed(2)}
                  </td>
                  <td style={{ padding: '0.5rem', textAlign: 'center' }}>
                    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '0.5rem' }}>
                      <button
                        aria-label="Decrease quantity"
                        disabled={upsertItem.isPending}
                        onClick={() => upsertItem.mutate({ bookId: item.bookId, quantity: item.quantity - 1 })}
                        style={{ width: 28, height: 28, cursor: 'pointer' }}
                      >
                        −
                      </button>
                      <span style={{ minWidth: 24, textAlign: 'center' }}>{item.quantity}</span>
                      <button
                        aria-label="Increase quantity"
                        disabled={upsertItem.isPending}
                        onClick={() => upsertItem.mutate({ bookId: item.bookId, quantity: item.quantity + 1 })}
                        style={{ width: 28, height: 28, cursor: 'pointer' }}
                      >
                        +
                      </button>
                    </div>
                  </td>
                  <td style={{ padding: '0.5rem', textAlign: 'right' }}>
                    ${((item.unitPrice * item.quantity) / 100).toFixed(2)}
                  </td>
                  <td style={{ padding: '0.5rem' }}>
                    <button
                      aria-label={`Remove ${item.title}`}
                      onClick={() => upsertItem.mutate({ bookId: item.bookId, quantity: 0 })}
                      style={{ color: '#c00', background: 'none', border: 'none', cursor: 'pointer' }}
                      disabled={upsertItem.isPending}
                    >
                      ✕
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
            <tfoot>
              <tr>
                <td colSpan={3} style={{ textAlign: 'right', fontWeight: 'bold', padding: '0.75rem 0.5rem 0.25rem' }}>
                  Subtotal:
                </td>
                <td style={{ textAlign: 'right', fontWeight: 'bold', padding: '0.75rem 0.5rem 0.25rem' }}>
                  ${((cart?.subtotal ?? 0) / 100).toFixed(2)}
                </td>
                <td />
              </tr>
            </tfoot>
          </table>

          <div style={{ marginTop: '1.25rem', display: 'flex', gap: '1rem', alignItems: 'center' }}>
            <button
              onClick={() => clearCart.mutate()}
              disabled={clearCart.isPending}
              style={{ color: '#c00', background: 'none', border: '1px solid #c00', padding: '0.4rem 1rem', borderRadius: 4, cursor: 'pointer' }}
            >
              Clear Cart
            </button>
            <div style={{ flex: 1 }} />
            <Link to="/checkout">
              <button style={{ padding: '0.65rem 2rem', fontSize: '1rem', cursor: 'pointer' }}>
                Proceed to Checkout →
              </button>
            </Link>
          </div>
        </>
      )}
    </div>
  );
}
