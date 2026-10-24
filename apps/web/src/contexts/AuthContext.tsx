import Keycloak from 'keycloak-js';
import React, { createContext, useContext, useEffect, useState } from 'react';
import type { AuthUser } from '@/types';

interface AuthContextValue {
  keycloak: Keycloak | null;
  user: AuthUser | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  login: () => void;
  logout: () => void;
  token: string | undefined;
  hasRole: (role: string) => boolean;
}

const AuthContext = createContext<AuthContextValue | null>(null);

const keycloakInstance = new Keycloak({
  url: import.meta.env.VITE_KEYCLOAK_URL ?? 'http://localhost:8080',
  realm: import.meta.env.VITE_KEYCLOAK_REALM ?? 'bookstore',
  clientId: import.meta.env.VITE_KEYCLOAK_CLIENT_ID ?? 'bookstore-web',
});

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [isLoading, setIsLoading] = useState(true);
  const [user, setUser] = useState<AuthUser | null>(null);

  useEffect(() => {
    keycloakInstance
      .init({ onLoad: 'check-sso', pkceMethod: 'S256' })
      .then((authenticated) => {
        if (authenticated && keycloakInstance.tokenParsed) {
          const parsed = keycloakInstance.tokenParsed as Record<string, unknown>;
          setUser({
            sub: String(parsed['sub'] ?? ''),
            email: parsed['email'] as string | undefined,
            preferredUsername: parsed['preferred_username'] as string | undefined,
            tenantId: parsed['tenant_id'] as string | undefined,
            roles: ((parsed['realm_access'] as { roles?: string[] })?.roles) ?? [],
          });
        }
      })
      .finally(() => setIsLoading(false));
  }, []);

  const login = () => keycloakInstance.login();
  const logout = () => keycloakInstance.logout({ redirectUri: window.location.origin });
  const hasRole = (role: string) => user?.roles.includes(role) ?? false;

  return (
    <AuthContext.Provider
      value={{
        keycloak: keycloakInstance,
        user,
        isAuthenticated: !!user,
        isLoading,
        login,
        logout,
        token: keycloakInstance.token,
        hasRole,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be used inside AuthProvider');
  return ctx;
}
