// TanStack Query hooks for Analytics domain
import { useQuery } from '@tanstack/react-query';
import {api, AnalyticsQuery} from '../api/client';

// Query keys
export const analyticsKeys = {
  all: ['analytics'] as const,
  listings: (query?: AnalyticsQuery) => [...analyticsKeys.all, 'listings', query] as const,
};

// Hooks
export function useListingsAnalytics(query?: AnalyticsQuery) {
  return useQuery({
    queryKey: analyticsKeys.listings(query),
    queryFn: () => api.analytics.listings(query),
    staleTime: 1000 * 60 * 5, // 5 minutes
  });
}

export function useListingAnalytics(listingId: string) {
  return useQuery({
    queryKey: [...analyticsKeys.all, 'listing', listingId],
    queryFn: async () => {
      const response = await api.analytics.listings({
        limit: 1,
        offset: 0,
        // Note: We'd need a specific endpoint for single listing, for now filter
      });
      return response.items[0];
    },
    enabled: !!listingId,
    staleTime: 1000 * 60 * 5,
  });
}