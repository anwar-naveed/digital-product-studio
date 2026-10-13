'use client';

import { useState, useCallback, useEffect } from 'react';
import { Loader2, RefreshCw, TrendingUp, DollarSign, Download, BarChart3 } from 'lucide-react';

import { Button } from '@/components/ui/Button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';
import { Input } from '@/components/ui/Input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/Select';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/Tabs';
import { api, type ListingAnalyticsResponse, type AnalyticsListResponse, type AnalyticsQuery, type Platform } from '@/api/client';

const platformIcons: Record<Platform, React.ReactNode> = {
  etsy: <span className="text-green-600">🛍️</span>,
  'lemon-squeezy': <span className="text-yellow-600">⚡</span>,
  printify: <span className="text-blue-600">🖨️</span>,
  gelato: <span className="text-purple-600">📦</span>,
};

interface AnalyticsTablesProps {
  initialQuery?: AnalyticsQuery;
}

export function AnalyticsTables({ initialQuery }: AnalyticsTablesProps) {
  const [query, setQuery] = useState<AnalyticsQuery>({
    limit: 50,
    offset: 0,
    ...initialQuery,
  });
  const [analytics, setAnalytics] = useState<ListingAnalyticsResponse[]>([]);
  const [total, setTotal] = useState(0);
  const [isLoading, setIsLoading] = useState(false);
  const [activeTab, setActiveTab] = useState<'performance' | 'run-cost'>('performance');

  const fetchAnalytics = useCallback(async () => {
    setIsLoading(true);
    try {
      const response: AnalyticsListResponse = await api.analytics.listings(query);
      setAnalytics(response.items);
      setTotal(response.total);
    } catch (error) {
      console.error('Failed to fetch analytics:', error);
    } finally {
      setIsLoading(false);
    }
  }, [query]);

  useEffect(() => {
    fetchAnalytics();
  }, [fetchAnalytics]);

  const handleFilterChange = (newQuery: Partial<AnalyticsQuery>) => {
    setQuery(prev => ({ ...prev, ...newQuery, offset: 0 }));
  };

  const handlePageChange = (newOffset: number) => {
    setQuery(prev => ({ ...prev, offset: newOffset }));
  };

  const formatNumber = (num: number) => {
    return new Intl.NumberFormat().format(num);
  };

  const formatCurrency = (str: string) => {
    const num = parseFloat(str);
    return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(num);
  };

  const formatPercent = (v: number | string) => `${Number(v).toFixed(1)}%`;

  return (
    <Card className="w-full">
      <CardHeader className="flex flex-row items-center justify-between">
        <div className="flex items-center gap-2">
          <BarChart3 className="h-5 w-5" />
          <CardTitle>Analytics</CardTitle>
        </div>
        <div className="flex items-center gap-4">
          <Tabs value={activeTab} onValueChange={setActiveTab} className="flex flex-row">
            <TabsList>
              <TabsTrigger value="performance">
                <TrendingUp className="mr-2 h-4 w-4" />
                Listing Performance
              </TabsTrigger>
              <TabsTrigger value="run-cost">
                <DollarSign className="mr-2 h-4 w-4" />
                Run Cost Analysis
              </TabsTrigger>
            </TabsList>
          </Tabs>
          <Select
            value={query.platform ?? ''}
            onValueChange={(value) => handleFilterChange({ platform: value as Platform || undefined })}
          >
            <SelectTrigger className="w-[180px]">
              <SelectValue placeholder="All platforms" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">All Platforms</SelectItem>
              <SelectItem value="etsy">Etsy</SelectItem>
              <SelectItem value="lemon-squeezy">Lemon Squeezy</SelectItem>
              <SelectItem value="printify">Printify</SelectItem>
              <SelectItem value="gelato">Gelato</SelectItem>
            </SelectContent>
          </Select>
          <Input
            placeholder="Filter by product ID..."
            value={query.product_id ?? ''}
            onChange={(e) => handleFilterChange({ product_id: e.target.value || undefined })}
            className="w-[200px]"
          />
          <Button
            variant="outline"
            size="icon"
            onClick={fetchAnalytics}
            disabled={isLoading}
            aria-label="Refresh analytics"
          >
            {isLoading ? <Loader2 className="h-4 w-4 animate-spin" /> : <RefreshCw className="h-4 w-4" />}
          </Button>
          <Button variant="outline" size="sm">
            <Download className="mr-2 h-4 w-4" />
            Export CSV
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {isLoading && analytics.length === 0 ? (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        ) : analytics.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 text-center">
            <BarChart3 className="h-12 w-12 text-muted-foreground mb-4" />
            <p className="text-muted-foreground">No analytics data found</p>
          </div>
        ) : activeTab === 'performance' ? (
          <PerformanceTable
            analytics={analytics}
            total={total}
            query={query}
            onPageChange={handlePageChange}
            formatNumber={formatNumber}
            formatCurrency={formatCurrency}
            formatPercent={formatPercent}
          />
        ) : (
          <RunCostTable
            analytics={analytics}
            total={total}
            query={query}
            onPageChange={handlePageChange}
            formatNumber={formatNumber}
            formatCurrency={formatCurrency}
            formatPercent={formatPercent}
          />
        )}
      </CardContent>
    </Card>
  );
}

function PerformanceTable({
  analytics,
  total,
  query,
  onPageChange,
  formatNumber,
  formatCurrency,
  formatPercent,
}: {
  analytics: ListingAnalyticsResponse[];
  total: number;
  query: AnalyticsQuery;
  onPageChange: (offset: number) => void;
  formatNumber: (num: number) => string;
  formatCurrency: (str: string) => string;
  formatPercent: (str: string) => string;
}) {
  return (
    <>
      <div className="overflow-x-auto">
        <table className="w-full" role="table">
          <thead>
            <tr className="border-b text-left text-sm text-muted-foreground">
              <th className="pb-3 font-medium">Listing</th>
              <th className="pb-3 font-medium">Product</th>
              <th className="pb-3 font-medium">Platform</th>
              <th className="pb-3 font-medium">State</th>
              <th className="pb-3 font-medium">Current Price</th>
              <th className="pb-3 font-medium">Views</th>
              <th className="pb-3 font-medium">Favorites</th>
              <th className="pb-3 font-medium">Sales</th>
              <th className="pb-3 font-medium">Revenue</th>
              <th className="pb-3 font-medium">Fees</th>
              <th className="pb-3 font-medium">Net</th>
              <th className="pb-3 font-medium">Margin</th>
              <th className="pb-3 font-medium">First Sale</th>
              <th className="pb-3 font-medium">Last Sale</th>
            </tr>
          </thead>
          <tbody>
            {analytics.map((item) => (
              <tr key={item.listing_id} className="border-b hover:bg-muted/50 transition-colors">
                <td className="py-3 font-mono text-xs">{item.platform_listing_id}</td>
                <td className="py-3 text-sm text-muted-foreground max-w-xs truncate">
                  {item.product_sku} - {item.product_title}
                </td>
                <td className="py-3">
                  <span className="flex items-center gap-1">
                    {platformIcons[item.platform]}
                    <span className="font-medium text-sm">{item.platform.replace('-', ' ').replace(/\b\w/g, l => l.toUpperCase())}</span>
                  </span>
                </td>
                <td className="py-3">
                  <Badge variant={item.state === 'live' ? 'live' : item.state as any} className="gap-1">
                    {item.state.charAt(0).toUpperCase() + item.state.slice(1)}
                  </Badge>
                </td>
                <td className="py-3 text-sm font-medium">{formatCurrency(item.current_price)}</td>
                <td className="py-3 text-sm">{formatNumber(item.views)}</td>
                <td className="py-3 text-sm">{formatNumber(item.favorites)}</td>
                <td className="py-3 text-sm font-medium">{formatNumber(item.sales_count)}</td>
                <td className="py-3 text-sm font-medium">{formatCurrency(item.total_revenue)}</td>
                <td className="py-3 text-sm text-muted-foreground">{formatCurrency(item.total_fees)}</td>
                <td className="py-3 text-sm font-medium text-green-600">{formatCurrency(item.total_net)}</td>
                <td className="py-3 text-sm font-medium">{formatPercent(item.margin_percent)}</td>
                <td className="py-3 text-sm text-muted-foreground">
                  {item.first_sale_at ? new Date(item.first_sale_at).toLocaleDateString() : '—'}
                </td>
                <td className="py-3 text-sm text-muted-foreground">
                  {item.last_sale_at ? new Date(item.last_sale_at).toLocaleDateString() : '—'}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Pagination */}
      {total > query.limit && (
        <div className="flex items-center justify-between mt-6 pt-4 border-t">
          <p className="text-sm text-muted-foreground">
            Showing {query.offset + 1} to {Math.min(query.offset + analytics.length, total)} of {total}
          </p>
          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => onPageChange(Math.max(0, query.offset - query.limit))}
              disabled={query.offset === 0}
            >
              Previous
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => onPageChange(query.offset + query.limit)}
              disabled={query.offset + analytics.length >= total}
            >
              Next
            </Button>
          </div>
        </div>
      )}
    </>
  );
}

function RunCostTable({
  analytics,
  total,
  query,
  onPageChange,
  formatNumber,
  formatCurrency,
  formatPercent: _formatPercent,
}: {
  analytics: ListingAnalyticsResponse[];
  total: number;
  query: AnalyticsQuery;
  onPageChange: (offset: number) => void;
  formatNumber: (num: number) => string;
  formatCurrency: (str: string) => string;
  formatPercent: (str: string) => string;
}) {
  // Group by product for run cost analysis
  const productMap = new Map<string, ListingAnalyticsResponse[]>();
  analytics.forEach(item => {
    const existing = productMap.get(item.product_id) || [];
    existing.push(item);
    productMap.set(item.product_id, existing);
  });

  return (
    <>
      <div className="overflow-x-auto">
        <table className="w-full" role="table">
          <thead>
            <tr className="border-b text-left text-sm text-muted-foreground">
              <th className="pb-3 font-medium">Product</th>
              <th className="pb-3 font-medium">Platforms</th>
              <th className="pb-3 font-medium">Total Listings</th>
              <th className="pb-3 font-medium">Live Listings</th>
              <th className="pb-3 font-medium">Total Sales</th>
              <th className="pb-3 font-medium">Gross Revenue</th>
              <th className="pb-3 font-medium">Platform Fees</th>
              <th className="pb-3 font-medium">Payoneer Fee (≤3%)</th>
              <th className="pb-3 font-medium">Net Revenue</th>
              <th className="pb-3 font-medium">Avg Margin</th>
            </tr>
          </thead>
          <tbody>
            {Array.from(productMap.entries()).map(([productId, listings]) => {
              const liveListings = listings.filter(l => l.state === 'live');
              const totalSales = listings.reduce((sum, l) => sum + l.sales_count, 0);
              const grossRevenue = listings.reduce((sum, l) => sum + parseFloat(l.total_revenue), 0);
              const totalFees = listings.reduce((sum, l) => sum + parseFloat(l.total_fees), 0);
              const totalNet = listings.reduce((sum, l) => sum + parseFloat(l.total_net), 0);
              const avgMargin = listings.length > 0
                ? listings.reduce((sum, l) => sum + parseFloat(l.margin_percent), 0) / listings.length
                : 0;
              const payoneerFee = grossRevenue * 0.03; // ≤3% per R9

              return (
                <tr key={productId} className="border-b hover:bg-muted/50 transition-colors">
                  <td className="py-3">
                    <div>
                      <p className="font-medium">{listings[0].product_sku}</p>
                      <p className="text-sm text-muted-foreground truncate max-w-xs">{listings[0].product_title}</p>
                    </div>
                  </td>
                  <td className="py-3">
                    <div className="flex flex-wrap gap-1">
                      {Array.from(new Set(listings.map(l => l.platform))).map(p => (
                        <Badge key={p} variant="outline" className="gap-1">
                          {platformIcons[p]}
                          {p.replace('-', ' ').replace(/\b\w/g, l => l.toUpperCase())}
                        </Badge>
                      ))}
                    </div>
                  </td>
                  <td className="py-3 text-sm">{listings.length}</td>
                  <td className="py-3 text-sm font-medium">{liveListings.length}</td>
                  <td className="py-3 text-sm">{formatNumber(totalSales)}</td>
                  <td className="py-3 text-sm font-medium">{formatCurrency(grossRevenue.toString())}</td>
                  <td className="py-3 text-sm text-muted-foreground">{formatCurrency(totalFees.toString())}</td>
                  <td className="py-3 text-sm text-muted-foreground">{formatCurrency(payoneerFee.toString())}</td>
                  <td className="py-3 text-sm font-medium text-green-600">{formatCurrency(totalNet.toString())}</td>
                  <td className="py-3 text-sm font-medium">{avgMargin.toFixed(1)}%</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      {/* Pagination */}
      {total > query.limit && (
        <div className="flex items-center justify-between mt-6 pt-4 border-t">
          <p className="text-sm text-muted-foreground">
            Showing {query.offset + 1} to {Math.min(query.offset + analytics.length, total)} of {total}
          </p>
          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => onPageChange(Math.max(0, query.offset - query.limit))}
              disabled={query.offset === 0}
            >
              Previous
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => onPageChange(query.offset + query.limit)}
              disabled={query.offset + analytics.length >= total}
            >
              Next
            </Button>
          </div>
        </div>
      )}
    </>
  );
}