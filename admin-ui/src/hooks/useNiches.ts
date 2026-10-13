// TanStack Query hooks for Niche domain
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {api, CreateNicheRequest, UpdateNicheRequest, ListNichesQuery} from '../api/client';

// Query keys
export const nicheKeys = {
  all: ['niches'] as const,
  lists: () => [...nicheKeys.all, 'list'] as const,
  list: (query?: ListNichesQuery) => [...nicheKeys.lists(), query] as const,
  details: () => [...nicheKeys.all, 'detail'] as const,
  detail: (id: string) => [...nicheKeys.details(), id] as const,
};

// Hooks
export function useNiches(query?: ListNichesQuery) {
  return useQuery({
    queryKey: nicheKeys.list(query),
    queryFn: () => api.niches.list(query),
    staleTime: 1000 * 60 * 5, // 5 minutes
  });
}

export function useNiche(id: string) {
  return useQuery({
    queryKey: nicheKeys.detail(id),
    queryFn: () => api.niches.get(id),
    enabled: !!id,
    staleTime: 1000 * 60 * 5,
  });
}

export function useCreateNiche() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (data: CreateNicheRequest) => api.niches.create(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: nicheKeys.lists() });
    },
  });
}

export function useUpdateNiche() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateNicheRequest }) => api.niches.update(id, data),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({ queryKey: nicheKeys.lists() });
      queryClient.invalidateQueries({ queryKey: nicheKeys.detail(id) });
    },
  });
}

export function useDeleteNiche() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (id: string) => api.niches.delete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: nicheKeys.lists() });
    },
  });
}