import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { usersApi } from '@/services/api';

export default function ProfilePage() {
  const qc = useQueryClient();

  const { data: profile } = useQuery({ queryKey: ['profile'], queryFn: () => usersApi.getProfile() });
  const { data: addresses } = useQuery({ queryKey: ['addresses'], queryFn: () => usersApi.listAddresses() });
  const { data: paymentMethods } = useQuery({ queryKey: ['paymentMethods'], queryFn: () => usersApi.listPaymentMethods() });

  const updateProfile = useMutation({
    mutationFn: usersApi.updateProfile,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['profile'] }),
  });

  const deleteAddress = useMutation({
    mutationFn: (id: string) => usersApi.deleteAddress(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['addresses'] }),
  });

  const deletePayment = useMutation({
    mutationFn: (id: string) => usersApi.deletePaymentMethod(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['paymentMethods'] }),
  });

  const [firstName, setFirstName] = useState(profile?.firstName ?? '');
  const [lastName, setLastName] = useState(profile?.lastName ?? '');
  const [phone, setPhone] = useState(profile?.phone ?? '');

  return (
    <div style={{ padding: '1rem', maxWidth: 600 }}>
      <h1>My Profile</h1>

      <section style={{ marginBottom: '2rem' }}>
        <h2>Personal Info</h2>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', maxWidth: 300 }}>
          <input placeholder="First name" value={firstName} onChange={(e) => setFirstName(e.target.value)} />
          <input placeholder="Last name" value={lastName} onChange={(e) => setLastName(e.target.value)} />
          <input placeholder="Phone" value={phone} onChange={(e) => setPhone(e.target.value)} />
          <button onClick={() => updateProfile.mutate({ firstName, lastName, phone })}>Save</button>
        </div>
      </section>

      <section style={{ marginBottom: '2rem' }}>
        <h2>Shipping Addresses</h2>
        {addresses?.map((addr) => (
          <div key={addr.id} style={{ border: '1px solid #eee', padding: '0.5rem', marginBottom: '0.5rem', borderRadius: 4 }}>
            <p style={{ margin: 0 }}>
              {addr.label && <strong>{addr.label}: </strong>}
              {addr.line1}{addr.line2 ? `, ${addr.line2}` : ''}, {addr.city}, {addr.postalCode}, {addr.country}
            </p>
            <button onClick={() => deleteAddress.mutate(addr.id)}>Remove</button>
          </div>
        ))}
      </section>

      <section>
        <h2>Payment Methods</h2>
        {paymentMethods?.map((pm) => (
          <div key={pm.id} style={{ border: '1px solid #eee', padding: '0.5rem', marginBottom: '0.5rem', borderRadius: 4 }}>
            <p style={{ margin: 0 }}>
              {pm.label} — {pm.methodType}
              {pm.lastFour && ` •••• ${pm.lastFour}`}
            </p>
            <button onClick={() => deletePayment.mutate(pm.id)}>Remove</button>
          </div>
        ))}
      </section>
    </div>
  );
}
