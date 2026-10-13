'use client';

import { useState, useCallback } from 'react';
import { Loader2, RefreshCw } from 'lucide-react';

import { Button } from '@/components/ui/Button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';


import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/Select';
import { CurationCard } from '@/components/CurationCard';
import { api, type CurationQueueItem, type CurationQueueQuery, type CurationQueueResponse } from '@/api/client';

interface CurationGridProps {
  initialQuery?: CurationQueueQuery;
}

export function CurationGrid({ initialQuery }: CurationGridProps) {
  const [query, setQuery] = useState<CurationQueueQuery>({
    limit: 20,
    offset: 0,
    decided: 'pending',
    ...initialQuery,
  });
  const [items, setItems] = useState<CurationQueueItem[]>([]);
  const [total, setTotal] = useState(0);
  const [isLoading, setIsLoading] = useState(false);
  const [_isMutating, setIsMutating] = useState(false);

  const fetchQueue = useCallback(async () => {
    setIsLoading(true);
    try {
      const response: CurationQueueResponse = await api.curation.queue(query);
      setItems(response.items);
      setTotal(response.total);
    } catch (error) {
      console.error('Failed to fetch curation queue:', error);
    } finally {
      setIsLoading(false);
    }
  }, [query]);

  const handleApprove = async (variantId: string) => {
    setIsMutating(true);
    try {
      await api.curation.approve(variantId, {});
      fetchQueue(); // Refetch to get updated queue
    } catch (error) {
      console.error('Failed to approve:', error);
    } finally {
      setIsMutating(false);
    }
  };

  const handleReject = async (variantId: string, rejectReason: string) => {
    setIsMutating(true);
    try {
      await api.curation.reject(variantId, { reject_reason: rejectReason as any });
      fetchQueue();
    } catch (error) {
      console.error('Failed to reject:', error);
    } finally {
      setIsMutating(false);
    }
  };

  const handleFilterChange = (newQuery: Partial<CurationQueueQuery>) => {
    setQuery(prev => ({ ...prev, ...newQuery, offset: 0 }));
  };

  const handlePageChange = (newOffset: number) => {
    setQuery(prev => ({ ...prev, offset: newOffset }));
  };

  return (
    <Card className="w-full">
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle>Curation Queue</CardTitle>
        <div className="flex items-center gap-4">
          <Select
            value={query.decided ?? 'pending'}
            onValueChange={(value) => handleFilterChange({ decided: value as any })}
          >
            <SelectTrigger className="w-[180px]">
              <SelectValue placeholder="Filter by status" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="pending">Pending</SelectItem>
              <SelectItem value="approved">Approved</SelectItem>
              <SelectItem value="rejected">Rejected</SelectItem>
            </SelectContent>
          </Select>
          <Button
            variant="outline"
            size="icon"
            onClick={fetchQueue}
            disabled={isLoading}
            aria-label="Refresh queue"
          >
            {isLoading ? <Loader2 className="h-4 w-4 animate-spin" /> : <RefreshCw className="h-4 w-4" />}
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {isLoading && items.length === 0 ? (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        ) : items.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 text-center">
            <p className="text-muted-foreground">No items in curation queue</p>
            <p className="text-sm text-muted-foreground">
              {query.decided === 'pending' ? 'All caught up!' : `No ${query.decided} items found.`}
            </p>
          </div>
        ) : (
          <>
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
              {items.map((item) => (
                <CurationCard
                  key={item.variant_id}
                  item={item}
                  onApprove={handleApprove}
                  onReject={handleReject}
                />
              ))}
            </div>

            {/* Pagination */}
            {total > query.limit && (
              <div className="flex items-center justify-between mt-6 pt-4 border-t">
                <p className="text-sm text-muted-foreground">
                  Showing {query.offset + 1} to {Math.min(query.offset + items.length, total)} of {total}
                </p>
                <div className="flex gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handlePageChange(Math.max(0, query.offset - query.limit))}
                    disabled={query.offset === 0}
                  >
                    Previous
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handlePageChange(query.offset + query.limit)}
                    disabled={query.offset + items.length >= total}
                  >
                    Next
                  </Button>
                </div>
              </div>
            )}
          </>
        )}
      </CardContent>
    </Card>
  );
}