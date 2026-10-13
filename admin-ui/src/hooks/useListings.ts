// TanStack Query hooks for Listings domain
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {api, ListListingsQuery, PublishRequest, DelistRequest, RepriceRequest} from '../api/client';

// Query keys
export const listingKeys = {
  all: ['listings'] as const,
  lists: () => [...listingKeys.all, 'list'] as const,
  list: (query?: ListListingsQuery) => [...listingKeys.lists(), query] as const,
};

// Hooks
export function useListings(query?: ListListingsQuery) {
  return useQuery({
    queryKey: listingKeys.list(query),
    queryFn: () => api.listings.list(query),
    staleTime: 1000 * 60 * 2, // 2 minutes
  });
}

export function usePublishListing() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ productId, data }: { productId: string; data: PublishRequest }) => 
      api.listings.publish(productId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: listingKeys.lists() });
    },
  });
}

export function useDelistListing() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ productId, data }: { productId: string; data: DelistRequest }) => 
      api.listings.delist(productId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: listingKeys.lists() });
    },
  });
}

export function useRepriceListing() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ productId, data }: { productId: string; data: RepriceRequest }) => 
      api.listings.reprice(productId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: listingKeys.lists() });
    },
  });
}