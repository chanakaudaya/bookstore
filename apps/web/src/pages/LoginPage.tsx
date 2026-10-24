import { useAuth } from '@/contexts/AuthContext';

export default function LoginPage() {
  const { login } = useAuth();
  return (
    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', height: '100vh' }}>
      <h1>Bookstore</h1>
      <p>Please sign in to continue.</p>
      <button onClick={login} style={{ padding: '0.75rem 2rem', fontSize: '1rem' }}>Sign In</button>
    </div>
  );
}
