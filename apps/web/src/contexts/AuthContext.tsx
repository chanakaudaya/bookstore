import Keycloak from 'keycloak-js';
import React, { createContext, useContext, useEffect, useRef, useState } from 'react';
import type { AuthUser } from '@/types';
import { setAuthToken } from '@/services/api';

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

/** How many seconds before expiry to trigger a proactive refresh. */
const REFRESH_BEFORE_EXPIRY_SEC = 60;
/** Interval (ms) at which we check whether the token needs refreshing. */
const REFRESH_CHECK_INTERVAL_MS = 30_000;

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [isLoading, setIsLoading] = useState(true);
  const [user, setUser] = useState<AuthUser | null>(null);
  const [token, setToken] = useState<string | undefined>(undefined);
  const refreshTimerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const applyToken = (kc: Keycloak) => {
    const t = kc.token;
    setToken(t);
    setAuthToken(t);
  };

  const extractUser = (kc: Keycloak): AuthUser | null => {
    const parsed = kc.tokenParsed as Record<string, unknown> | undefined;
    if (!parsed) return null;
    return {
      sub: String(parsed['sub'] ?? ''),
      email: parsed['email'] as string | undefined,
      preferredUsername: parsed['preferred_username'] as string | undefined,
      tenantId: parsed['tenant_id'] as string | undefined,
      roles: ((parsed['realm_access'] as { roles?: string[] })?.roles) ?? [],
    };
  };

  const startRefreshTimer = (kc: Keycloak) => {
    if (refreshTimerRef.current) clearInterval(refreshTimerRef.current);
    refreshTimerRef.current = setInterval(async () => {
      try {
        const refreshed = await kc.updateToken(REFRESH_BEFORE_EXPIRY_SEC);
        if (refreshed) {
          applyToken(kc);
        }
      } catch {
        // Refresh failed — session likely expired; redirect to login
        kc.logout({ redirectUri: window.location.origin });
      }
    }, REFRESH_CHECK_INTERVAL_MS);
  };

  useEffect(() => {
    keycloakInstance
      .init({ onLoad: 'check-sso', pkceMethod: 'S256' })
      .then((authenticated) => {
        if (authenticated) {
          applyToken(keycloakInstance);
          setUser(extractUser(keycloakInstance));
          startRefreshTimer(keycloakInstance);
        }
      })
      .finally(() => setIsLoading(false));

    return () => {
      if (refreshTimerRef.current) clearInterval(refreshTimerRef.current);
    };
  }, []);

  const login = () => keycloakInstance.login();
  const logout = () => {
    if (refreshTimerRef.current) clearInterval(refreshTimerRef.current);
    setAuthToken(undefined);
    keycloakInstance.logout({ redirectUri: window.location.origin });
  };
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
        token,
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
