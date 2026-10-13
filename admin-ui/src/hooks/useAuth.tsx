// Auth hook for managing authentication state
import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { authApi } from '../api/client';

interface AuthContextType {
  isAuthenticated: boolean;
  isLoading: boolean;
  login: (username: string, password: string) => Promise<void>;
  logout: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    // Check if token exists on mount
    const token = authApi.isAuthenticated();
    setIsAuthenticated(token);
    setIsLoading(false);
  }, []);

  const login = async (username: string, password: string) => {
    await authApi.login(username, password);
    setIsAuthenticated(true);
  };

  const logout = () => {
    authApi.logout();
    setIsAuthenticated(false);
  };

  return (
    <AuthContext.Provider value={{ isAuthenticated, isLoading, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}