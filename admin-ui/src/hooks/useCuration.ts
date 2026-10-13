// TanStack Query hooks for Curation domain
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {api, CurationQueueQuery, ApproveCurationRequest, RejectCurationRequest} from '../api/client';

// Query keys
export const curationKeys = {
  all: ['curation'] as const,
  queue: (query?: CurationQueueQuery) => [...curationKeys.all, 'queue', query] as const,
};

// Hooks
export function useCurationQueue(query?: CurationQueueQuery) {
  return useQuery({
    queryKey: curationKeys.queue(query),
    queryFn: () => api.curation.queue(query),
    staleTime: 1000 * 30 * 1, // 30 seconds
  });
}

export function useApproveCuration() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ variantId, data }: { variantId: string; data: ApproveCurationRequest }) => 
      api.curation.approve(variantId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: curationKeys.all });
    },
  });
}

export function useRejectCuration() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ variantId, data }: { variantId: string; data: RejectCurationRequest }) => 
      api.curation.reject(variantId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: curationKeys.all });
    },
  });
}