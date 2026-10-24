import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter, Routes, Route } from 'react-router-dom';
import { ProtectedRoute } from '@/components/common/ProtectedRoute';

// Mock the AuthContext
vi.mock('@/contexts/AuthContext', () => ({
  useAuth: vi.fn(),
}));

import { useAuth } from '@/contexts/AuthContext';
const mockUseAuth = useAuth as ReturnType<typeof vi.fn>;

function TestPage() {
  return <div>Protected content</div>;
}

function UnauthorizedPage() {
  return <div>Home page</div>;
}

function renderWithRouter(
  isAuthenticated: boolean,
  isLoading: boolean,
  role?: string
) {
  const hasRole = (r: string) => r === role;
  mockUseAuth.mockReturnValue({
    isAuthenticated,
    isLoading,
    hasRole,
    login: vi.fn(),
    logout: vi.fn(),
    user: isAuthenticated ? { sub: 'u1', roles: role ? [role] : [] } : null,
    token: undefined,
    keycloak: null,
  });

  return render(
    <MemoryRouter initialEntries={['/protected']}>
      <Routes>
        <Route path="/" element={<UnauthorizedPage />} />
        <Route element={<ProtectedRoute />}>
          <Route path="/protected" element={<TestPage />} />
        </Route>
      </Routes>
    </MemoryRouter>
  );
}

describe('ProtectedRoute', () => {
  beforeEach(() => { vi.clearAllMocks(); });

  it('shows loading state while auth is resolving', () => {
    renderWithRouter(false, true);
    expect(screen.getByText('Loading...')).toBeInTheDocument();
  });

  it('renders protected content when authenticated', () => {
    renderWithRouter(true, false);
    expect(screen.getByText('Protected content')).toBeInTheDocument();
  });

  it('calls login when not authenticated', () => {
    const loginMock = vi.fn();
    mockUseAuth.mockReturnValue({
      isAuthenticated: false,
      isLoading: false,
      hasRole: () => false,
      login: loginMock,
      logout: vi.fn(),
      user: null,
      token: undefined,
      keycloak: null,
    });

    render(
      <MemoryRouter initialEntries={['/protected']}>
        <Routes>
          <Route element={<ProtectedRoute />}>
            <Route path="/protected" element={<TestPage />} />
          </Route>
        </Routes>
      </MemoryRouter>
    );

    expect(loginMock).toHaveBeenCalled();
  });
});

describe('ProtectedRoute with role', () => {
  beforeEach(() => { vi.clearAllMocks(); });

  it('renders content when user has required role', () => {
    mockUseAuth.mockReturnValue({
      isAuthenticated: true,
      isLoading: false,
      hasRole: (r: string) => r === 'tenant_admin',
      login: vi.fn(),
      logout: vi.fn(),
      user: { sub: 'u1', roles: ['tenant_admin'] },
      token: undefined,
      keycloak: null,
    });

    render(
      <MemoryRouter initialEntries={['/admin']}>
        <Routes>
          <Route path="/" element={<UnauthorizedPage />} />
          <Route element={<ProtectedRoute requiredRole="tenant_admin" />}>
            <Route path="/admin" element={<div>Admin content</div>} />
          </Route>
        </Routes>
      </MemoryRouter>
    );

    expect(screen.getByText('Admin content')).toBeInTheDocument();
  });

  it('redirects to / when user lacks required role', () => {
    mockUseAuth.mockReturnValue({
      isAuthenticated: true,
      isLoading: false,
      hasRole: () => false,
      login: vi.fn(),
      logout: vi.fn(),
      user: { sub: 'u1', roles: ['customer'] },
      token: undefined,
      keycloak: null,
    });

    render(
      <MemoryRouter initialEntries={['/admin']}>
        <Routes>
          <Route path="/" element={<UnauthorizedPage />} />
          <Route element={<ProtectedRoute requiredRole="tenant_admin" />}>
            <Route path="/admin" element={<div>Admin content</div>} />
          </Route>
        </Routes>
      </MemoryRouter>
    );

    expect(screen.getByText('Home page')).toBeInTheDocument();
    expect(screen.queryByText('Admin content')).not.toBeInTheDocument();
  });
});
