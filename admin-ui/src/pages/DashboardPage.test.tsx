// Vitest spec for DashboardPage component
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { DashboardPage } from '../pages/DashboardPage';


// Mock hooks
vi.mock('../hooks/useNiches', () => ({
  useNiches: vi.fn(() => ({
    data: { niches: [{ id: '1', name: 'Test', vertical: 'test', competition_score: '0.5', created_at: '2024-01-01', demand_signals: {} }], total: 1, limit: 20, offset: 0 },
    isLoading: false,
    error: null,
  })),
}));

vi.mock('../hooks/useRuns', () => ({
  useRuns: vi.fn(() => ({
    data: { runs: [{ id: '1', niche_id: '1', status: 'running', target_count: 10, params: {}, created_at: '2024-01-01', started_at: null, completed_at: null }], total: 1, limit: 20, offset: 0 },
    isLoading: false,
    error: null,
  })),
}));

vi.mock('../hooks/useProducts', () => ({
  useProducts: vi.fn(() => ({
    data: { products: [{ id: '1', sku: 'TEST-001', title: 'Test Product', license_tier: 'personal', price: '29.99', s3_prefix: 'test/', created_at: '2024-01-01' }], total: 1, limit: 20, offset: 0 },
    isLoading: false,
    error: null,
  })),
}));

vi.mock('../hooks/useListings', () => ({
  useListings: vi.fn(() => ({
    data: { listings: [{ id: '1', product_id: '1', platform: 'etsy', platform_listing_id: '123', state: 'live', price: '29.99', fee_table_id: null, created_at: '2024-01-01', updated_at: '2024-01-01' }], total: 1, limit: 20, offset: 0 },
    isLoading: false,
    error: null,
  })),
}));

vi.mock('../components/LoadingSpinner', () => ({
  LoadingSpinner: () => <div data-testid="loading-spinner">Loading...</div>,
}));

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
};

describe('DashboardPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders dashboard title', () => {
    render(<DashboardPage />, { wrapper: createWrapper() });
    expect(screen.getByText('Dashboard')).toBeInTheDocument();
    expect(screen.getByText('Overview of your digital product studio pipeline')).toBeInTheDocument();
  });

  it('renders stats cards with data', () => {
    render(<DashboardPage />, { wrapper: createWrapper() });
    
    expect(screen.getByText('Total Niches')).toBeInTheDocument();
    expect(screen.getAllByText('1').length).toBeGreaterThan(0); // total niches (value may repeat)
    expect(screen.getByText('Active Runs')).toBeInTheDocument();
    expect(screen.getByText('Products')).toBeInTheDocument();
    expect(screen.getByText('Listings')).toBeInTheDocument();
  });

  it('renders quick actions section', () => {
    render(<DashboardPage />, { wrapper: createWrapper() });
    expect(screen.getByText('Quick Actions')).toBeInTheDocument();
    expect(screen.getByText('Create New Niche')).toBeInTheDocument();
    expect(screen.getByText('Spawn Production Run')).toBeInTheDocument();
    expect(screen.getByText('Review Curation Queue')).toBeInTheDocument();
  });

  it('renders recent activity section', () => {
    render(<DashboardPage />, { wrapper: createWrapper() });
    expect(screen.getByText('Recent Activity')).toBeInTheDocument();
  });
});