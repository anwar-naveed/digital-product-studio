// TanStack Query hooks for Runs domain
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {api, CreateRunRequest, ListRunsQuery} from '../api/client';

// Query keys
export const runKeys = {
  all: ['runs'] as const,
  lists: () => [...runKeys.all, 'list'] as const,
  list: (query?: ListRunsQuery) => [...runKeys.lists(), query] as const,
  details: () => [...runKeys.all, 'detail'] as const,
  detail: (id: string) => [...runKeys.details(), id] as const,
};

// Hooks
export function useRuns(query?: ListRunsQuery) {
  return useQuery({
    queryKey: runKeys.list(query),
    queryFn: () => api.runs.list(query),
    staleTime: 1000 * 30 * 1, // 30 seconds for runs (more dynamic)
  });
}

export function useRun(id: string) {
  return useQuery({
    queryKey: runKeys.detail(id),
    queryFn: () => api.runs.get(id),
    enabled: !!id,
    staleTime: 1000 * 30 * 1,
  });
}

export function useCreateRun() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (data: CreateRunRequest) => api.runs.create(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: runKeys.lists() });
    },
  });
}

export function useCancelRun() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (id: string) => api.runs.cancel(id),
    onSuccess: (_, id) => {
      queryClient.invalidateQueries({ queryKey: runKeys.lists() });
      queryClient.invalidateQueries({ queryKey: runKeys.detail(id) });
    },
  });
}