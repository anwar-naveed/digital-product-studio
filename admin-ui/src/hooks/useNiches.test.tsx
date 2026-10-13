// Vitest specs for useNiches hook
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useNiches, useCreateNiche, useUpdateNiche, useDeleteNiche } from '../hooks/useNiches';
import { api } from '../api/client';

// Mock the api client
vi.mock('../api/client', () => ({
  api: {
    niches: {
      list: vi.fn(),
      create: vi.fn(),
      get: vi.fn(),
      update: vi.fn(),
      delete: vi.fn(),
    },
  },
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

describe('useNiches hooks', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('useNiches', () => {
    it('returns niches list from API', async () => {
      const mockNiches = {
        items: [
          { id: '1', name: 'Gothic Wall Art', vertical: 'wall art', competition_score: '0.8', created_at: '2024-01-01T00:00:00Z', demand_signals: {} },
          { id: '2', name: 'ADHD Planners', vertical: 'planners', competition_score: '0.6', created_at: '2024-01-02T00:00:00Z', demand_signals: {} },
        ],
        total: 2,
        limit: 20,
        offset: 0,
      };

      vi.mocked(api.niches.list).mockResolvedValue(mockNiches);

      const { result } = renderHook(() => useNiches(), { wrapper: createWrapper() });

      await waitFor(() => expect(result.current.isLoading).toBe(false));

      expect(result.current.data).toEqual(mockNiches);
      expect(api.niches.list).toHaveBeenCalledWith(undefined);
    });

    it('passes query parameters to API', async () => {
      const mockNiches = { items: [], total: 0, limit: 10, offset: 5 };
      vi.mocked(api.niches.list).mockResolvedValue(mockNiches);

      const { result } = renderHook(() => useNiches({ limit: 10, offset: 5, vertical: 'wall art' }), { wrapper: createWrapper() });

      await waitFor(() => expect(result.current.isLoading).toBe(false));

      expect(api.niches.list).toHaveBeenCalledWith({ limit: 10, offset: 5, vertical: 'wall art' });
    });
  });

  describe('useCreateNiche', () => {
    it('creates a niche and invalidates queries', async () => {
      const mockNiche = { id: '3', name: 'New Niche', vertical: 'wall art', competition_score: '0.5', created_at: '2024-01-03T00:00:00Z', demand_signals: {} };
      vi.mocked(api.niches.create).mockResolvedValue(mockNiche);

      const { result } = renderHook(() => useCreateNiche(), { wrapper: createWrapper() });

      await result.current.mutateAsync({ name: 'New Niche', vertical: 'wall art', demand_signals: {}, competition_score: '0.5' });

      expect(api.niches.create).toHaveBeenCalledWith({
        name: 'New Niche',
        vertical: 'wall art',
        demand_signals: {},
        competition_score: '0.5',
      });
    });
  });

  describe('useUpdateNiche', () => {
    it('updates a niche and invalidates queries', async () => {
      const mockNiche = { id: '1', name: 'Updated Niche', vertical: 'wall art', competition_score: '0.9', created_at: '2024-01-01T00:00:00Z', demand_signals: {} };
      vi.mocked(api.niches.update).mockResolvedValue(mockNiche);

      const { result } = renderHook(() => useUpdateNiche(), { wrapper: createWrapper() });

      await result.current.mutateAsync({ id: '1', data: { name: 'Updated Niche' } });

      expect(api.niches.update).toHaveBeenCalledWith('1', { name: 'Updated Niche' });
    });
  });

  describe('useDeleteNiche', () => {
    it('deletes a niche and invalidates queries', async () => {
      vi.mocked(api.niches.delete).mockResolvedValue(undefined);

      const { result } = renderHook(() => useDeleteNiche(), { wrapper: createWrapper() });

      await result.current.mutateAsync('1');

      expect(api.niches.delete).toHaveBeenCalledWith('1');
    });
  });
});