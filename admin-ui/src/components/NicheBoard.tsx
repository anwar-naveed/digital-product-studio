'use client';

import { useState, useCallback, useEffect } from 'react';
import { Loader2, RefreshCw, Search, TrendingUp, BarChart3, AlertTriangle } from 'lucide-react';

import { Button } from '@/components/ui/Button';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';
import { Input } from '@/components/ui/Input';

import { SpawnRunDialog } from './SpawnRunDialog';
import { api, type NicheResponse, type NicheListResponse, type ListNichesQuery } from '@/api/client';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/Select';

interface NicheBoardProps {
  initialQuery?: ListNichesQuery;
}

export function NicheBoard({ initialQuery }: NicheBoardProps) {
  const [query, setQuery] = useState<ListNichesQuery>({
    limit: 20,
    offset: 0,
    ...initialQuery,
  });
  const [niches, setNiches] = useState<NicheResponse[]>([]);
  const [total, setTotal] = useState(0);
  const [isLoading, setIsLoading] = useState(false);

  const fetchNiches = useCallback(async () => {
    setIsLoading(true);
    try {
      const response: NicheListResponse = await api.niches.list(query);
      setNiches(response.items);
      setTotal(response.total);
    } catch (error) {
      console.error('Failed to fetch niches:', error);
    } finally {
      setIsLoading(false);
    }
  }, [query]);

  useEffect(() => {
    fetchNiches();
  }, [fetchNiches]);

  const handleFilterChange = (newQuery: Partial<ListNichesQuery>) => {
    setQuery(prev => ({ ...prev, ...newQuery, offset: 0 }));
  };

  const handlePageChange = (newOffset: number) => {
    setQuery(prev => ({ ...prev, offset: newOffset }));
  };

  const formatDemandSignals = (signals: Record<string, unknown>) => {
    return Object.entries(signals).map(([key, value]) => (
      <Badge key={key} variant="outline" className="text-xs">
        {key}: {String(value)}
      </Badge>
    ));
  };

  const formatCompetition = (score: string) => {
    const num = parseFloat(score);
    if (num < 0.3) return 'Low';
    if (num < 0.6) return 'Medium';
    return 'High';
  };

  return (
    <Card className="w-full">
      <CardHeader className="flex flex-row items-center justify-between">
        <div className="flex items-center gap-2">
          <BarChart3 className="h-5 w-5" />
          <CardTitle>Niche Board</CardTitle>
        </div>
        <div className="flex items-center gap-4">
          <Select
            value={query.vertical ?? ''}
            onValueChange={(value) => handleFilterChange({ vertical: value || undefined })}
          >
            <SelectTrigger className="w-[180px]">
              <SelectValue placeholder="All verticals" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">All Verticals</SelectItem>
              <SelectItem value="wall-art">Wall Art</SelectItem>
              <SelectItem value="home-decor">Home Decor</SelectItem>
              <SelectItem value="apparel">Apparel</SelectItem>
            </SelectContent>
          </Select>
          <Input
            placeholder="Search niches..."
            value={''}
            onChange={(_e) => handleFilterChange({ limit: 20, offset: 0 })}
            className="w-[250px]"
            leftIcon={<Search className="h-4 w-4" />}
          />
          <Button
            variant="outline"
            size="icon"
            onClick={fetchNiches}
            disabled={isLoading}
            aria-label="Refresh niches"
          >
            {isLoading ? <Loader2 className="h-4 w-4 animate-spin" /> : <RefreshCw className="h-4 w-4" />}
          </Button>
          <SpawnRunDialog onSuccess={fetchNiches} />
        </div>
      </CardHeader>
      <CardContent>
        {isLoading && niches.length === 0 ? (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        ) : niches.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 text-center">
            <BarChart3 className="h-12 w-12 text-muted-foreground mb-4" />
            <p className="text-muted-foreground">No niches found</p>
            <p className="text-sm text-muted-foreground">
              Create niches to start spawning production runs
            </p>
          </div>
        ) : (
          <>
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
              {niches.map((niche) => (
                <Card key={niche.id} className="hover:shadow-md transition-shadow">
                  <CardHeader className="pb-3">
                    <div className="flex items-start justify-between">
                      <div className="flex-1 min-w-0">
                        <CardTitle className="truncate">{niche.name}</CardTitle>
                        <CardDescription className="text-xs">
                          {niche.vertical} • {formatCompetition(niche.competition_score)} competition
                        </CardDescription>
                      </div>
                      <Badge variant="secondary">{niche.vertical}</Badge>
                    </div>
                  </CardHeader>
                  <CardContent className="space-y-3 pt-0">
                    {/* Demand Signals */}
                    <div>
                      <p className="text-xs font-medium text-muted-foreground mb-1 flex items-center gap-1">
                        <TrendingUp className="h-3 w-3" />
                        Demand Signals
                      </p>
                      <div className="flex flex-wrap gap-1">
                        {formatDemandSignals(niche.demand_signals)}
                      </div>
                    </div>

                    {/* Competition Score */}
                    <div className="space-y-1">
                      <p className="text-xs font-medium text-muted-foreground flex items-center gap-1">
                        <AlertTriangle className="h-3 w-3" />
                        Competition Score: {niche.competition_score}
                      </p>
                      <div className="h-2 bg-muted rounded-full overflow-hidden">
                        <div
                          className="h-full bg-primary transition-all"
                          style={{ width: `${Math.min(parseFloat(niche.competition_score) * 100, 100)}%` }}
                        />
                      </div>
                    </div>

                    {/* Spawn Run CTA */}
                    <SpawnRunDialog defaultNicheId={niche.id} onSuccess={fetchNiches} />
                  </CardContent>
                </Card>
              ))}
            </div>

            {/* Pagination */}
            {total > query.limit && (
              <div className="flex items-center justify-between mt-6 pt-4 border-t">
                <p className="text-sm text-muted-foreground">
                  Showing {query.offset + 1} to {Math.min(query.offset + niches.length, total)} of {total}
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
                    disabled={query.offset + niches.length >= total}
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