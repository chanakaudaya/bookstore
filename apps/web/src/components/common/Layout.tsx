import { Link, Outlet } from 'react-router-dom';
import { useAuth } from '@/contexts/AuthContext';

const navStyle: React.CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  gap: '1.5rem',
  padding: '0.75rem 1.5rem',
  background: '#1a1a2e',
  color: '#fff',
};

const linkStyle: React.CSSProperties = {
  color: '#ccc',
  textDecoration: 'none',
  fontSize: '0.9rem',
};

const activeLinkStyle: React.CSSProperties = {
  ...linkStyle,
  color: '#fff',
  fontWeight: 600,
};

export function Layout() {
  const { user, isAuthenticated, logout, hasRole } = useAuth();

  return (
    <>
      <nav style={navStyle}>
        <Link to="/" style={{ ...activeLinkStyle, fontSize: '1.1rem' }}>
          📚 Bookstore
        </Link>

        <div style={{ display: 'flex', gap: '1rem', flex: 1 }}>
          <Link to="/" style={linkStyle}>Catalog</Link>
          {isAuthenticated && (
            <>
              <Link to="/cart" style={linkStyle}>Cart</Link>
              <Link to="/orders" style={linkStyle}>My Orders</Link>
              <Link to="/profile" style={linkStyle}>Profile</Link>
            </>
          )}
          {hasRole('tenant_admin') && (
            <>
              <span style={{ color: '#555' }}>|</span>
              <Link to="/admin/catalog" style={linkStyle}>Admin: Catalog</Link>
              <Link to="/admin/categories" style={linkStyle}>Admin: Categories</Link>
              <Link to="/admin/orders" style={linkStyle}>Admin: Orders</Link>
            </>
          )}
        </div>

        <div>
          {isAuthenticated ? (
            <span style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
              <span style={{ fontSize: '0.85rem', color: '#aaa' }}>
                {user?.preferredUsername ?? user?.email}
              </span>
              <button
                onClick={logout}
                style={{
                  background: 'transparent',
                  border: '1px solid #555',
                  color: '#ccc',
                  padding: '0.25rem 0.75rem',
                  borderRadius: 4,
                  cursor: 'pointer',
                  fontSize: '0.85rem',
                }}
              >
                Sign out
              </button>
            </span>
          ) : (
            <Link to="/login" style={activeLinkStyle}>Sign in</Link>
          )}
        </div>
      </nav>

      <main style={{ minHeight: 'calc(100vh - 52px)', background: '#fafafa' }}>
        <Outlet />
      </main>
    </>
  );
}
