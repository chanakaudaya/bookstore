import { useQuery, useMutation } from '@tanstack/react-query';
import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { v4 as uuidv4 } from 'uuid';
import { usersApi, checkoutApi } from '@/services/api';

export default function CheckoutPage() {
  const navigate = useNavigate();
  const [selectedAddress, setSelectedAddress] = useState<string>('');
  const [selectedPayment, setSelectedPayment] = useState<string>('');
  const [error, setError] = useState<string | null>(null);

  const { data: addresses } = useQuery({
    queryKey: ['addresses'],
    queryFn: () => usersApi.listAddresses(),
  });

  const { data: paymentMethods } = useQuery({
    queryKey: ['paymentMethods'],
    queryFn: () => usersApi.listPaymentMethods(),
  });

  const doCheckout = useMutation({
    mutationFn: () =>
      checkoutApi.checkout({
        idempotencyKey: uuidv4(),
        shippingAddressId: selectedAddress,
        paymentMethodId: selectedPayment,
      }),
    onSuccess: (result) => {
      navigate(`/orders/${result.orderId}`);
    },
    onError: () => {
      setError('Checkout failed. Please try again.');
    },
  });

  const canSubmit = selectedAddress && selectedPayment && !doCheckout.isPending;

  return (
    <div style={{ padding: '1rem', maxWidth: 500 }}>
      <h1>Checkout</h1>

      <section style={{ marginBottom: '1.5rem' }}>
        <h2>Shipping Address</h2>
        {addresses?.length === 0 && (
          <p>No addresses saved. <a href="/profile">Add one in your profile.</a></p>
        )}
        {addresses?.map((addr) => (
          <label key={addr.id} style={{ display: 'block', marginBottom: '0.5rem' }}>
            <input
              type="radio"
              name="address"
              value={addr.id}
              checked={selectedAddress === addr.id}
              onChange={() => setSelectedAddress(addr.id)}
            />{' '}
            {addr.label && <strong>{addr.label}: </strong>}
            {addr.line1}, {addr.city}, {addr.postalCode}
          </label>
        ))}
      </section>

      <section style={{ marginBottom: '1.5rem' }}>
        <h2>Payment Method</h2>
        {paymentMethods?.length === 0 && (
          <p>No payment methods saved. <a href="/profile">Add one in your profile.</a></p>
        )}
        {paymentMethods?.map((pm) => (
          <label key={pm.id} style={{ display: 'block', marginBottom: '0.5rem' }}>
            <input
              type="radio"
              name="payment"
              value={pm.id}
              checked={selectedPayment === pm.id}
              onChange={() => setSelectedPayment(pm.id)}
            />{' '}
            {pm.label} {pm.lastFour && `(•••• ${pm.lastFour})`}
          </label>
        ))}
      </section>

      {error && <p style={{ color: 'red' }}>{error}</p>}

      <button
        disabled={!canSubmit}
        onClick={() => doCheckout.mutate()}
        style={{ padding: '0.75rem 2rem', fontSize: '1rem' }}
      >
        {doCheckout.isPending ? 'Placing Order...' : 'Place Order'}
      </button>
    </div>
  );
}
