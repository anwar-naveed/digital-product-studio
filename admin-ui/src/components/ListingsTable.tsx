'use client';

import { useState, useCallback, useEffect } from 'react';
import { X, Loader2, RefreshCw, DollarSign, TrendingUp, AlertTriangle, Trash2, CheckCircle } from 'lucide-react';

import { Button } from '@/components/ui/Button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';
import { Input } from '@/components/ui/Input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/Select';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/Dialog';
import { api, type ListingResponse, type ListingListResponse, type ListListingsQuery, type ListingState, type Platform, type RepriceRequest, type RepriceResponse, type FeePreview } from '@/api/client';

const stateVariants: Record<ListingState | string, string> = {
  draft: 'draft',
  publishing: 'publishing',
  live: 'live',
  updating: 'updating',
  failed: 'failed',
  suspended: 'suspended',
  delisting: 'delisting',
  delisted: 'delisted',
};

const stateIcons: Record<ListingState | string, React.ReactNode> = {
  draft: <span className="w-2 h-2 rounded-full bg-gray-400" />,
  publishing: <Loader2 className="h-3 w-3 animate-spin" />,
  live: <CheckCircle className="h-3 w-3" />,
  updating: <Loader2 className="h-3 w-3 animate-spin" />,
  failed: <AlertTriangle className="h-3 w-3" />,
  suspended: <span className="w-2 h-2 rounded-full bg-orange-400" />,
  delisting: <Loader2 className="h-3 w-3 animate-spin" />,
  delisted: <span className="w-2 h-2 rounded-full bg-gray-400" />,
};

const platformIcons: Record<Platform, React.ReactNode> = {
  etsy: <span className="text-green-600">🛍️</span>,
  'lemon-squeezy': <span className="text-yellow-600">⚡</span>,
  printify: <span className="text-blue-600">🖨️</span>,
  gelato: <span className="text-purple-600">📦</span>,
};

const formatState = (state: string) => {
  return state.charAt(0).toUpperCase() + state.slice(1).replace('_', ' ');
};

const formatPlatform = (platform: string) => {
  return platform.replace('-', ' ').replace(/\b\w/g, l => l.toUpperCase());
};

interface ListingsTableProps {
  initialQuery?: ListListingsQuery;
}

export function ListingsTable({ initialQuery }: ListingsTableProps) {
  const [query, setQuery] = useState<ListListingsQuery>({
    limit: 20,
    offset: 0,
    ...initialQuery,
  });
  const [listings, setListings] = useState<ListingResponse[]>([]);
  const [total, setTotal] = useState(0);
  const [isLoading, setIsLoading] = useState(false);
  const [selectedListing, setSelectedListing] = useState<ListingResponse | null>(null);
  const [showRepriceDialog, setShowRepriceDialog] = useState(false);

  const fetchListings = useCallback(async () => {
    setIsLoading(true);
    try {
      const response: ListingListResponse = await api.listings.list(query);
      setListings(response.items);
      setTotal(response.total);
    } catch (error) {
      console.error('Failed to fetch listings:', error);
    } finally {
      setIsLoading(false);
    }
  }, [query]);

  useEffect(() => {
    fetchListings();
  }, [fetchListings]);

  const handleFilterChange = (newQuery: Partial<ListListingsQuery>) => {
    setQuery(prev => ({ ...prev, ...newQuery, offset: 0 }));
  };

  const handlePageChange = (newOffset: number) => {
    setQuery(prev => ({ ...prev, offset: newOffset }));
  };

  const handleDelist = async (listing: ListingResponse) => {
    if (!confirm(`Delist "${listing.platform_listing_id}" from ${formatPlatform(listing.platform)}?`)) return;
    try {
      await api.listings.delist(listing.product_id, { platform: listing.platform });
      fetchListings();
    } catch (err) {
      console.error('Failed to delist:', err);
      alert('Failed to delist listing');
    }
  };

  const handleReprice = (listing: ListingResponse) => {
    setSelectedListing(listing);
    setShowRepriceDialog(true);
  };

  const handleRepriceSuccess = () => {
    setShowRepriceDialog(false);
    setSelectedListing(null);
    fetchListings();
  };

  return (
    <Card className="w-full">
      <CardHeader className="flex flex-row items-center justify-between">
        <div className="flex items-center gap-2">
          <TrendingUp className="h-5 w-5" />
          <CardTitle>Listings Management</CardTitle>
        </div>
        <div className="flex items-center gap-4">
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
          <Select
            value={query.state ?? ''}
            onValueChange={(value) => handleFilterChange({ state: value as ListingState || undefined })}
          >
            <SelectTrigger className="w-[180px]">
              <SelectValue placeholder="All states" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">All States</SelectItem>
              <SelectItem value="draft">Draft</SelectItem>
              <SelectItem value="publishing">Publishing</SelectItem>
              <SelectItem value="live">Live</SelectItem>
              <SelectItem value="updating">Updating</SelectItem>
              <SelectItem value="failed">Failed</SelectItem>
              <SelectItem value="suspended">Suspended</SelectItem>
              <SelectItem value="delisting">Delisting</SelectItem>
              <SelectItem value="delisted">Delisted</SelectItem>
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
            onClick={fetchListings}
            disabled={isLoading}
            aria-label="Refresh listings"
          >
            {isLoading ? <Loader2 className="h-4 w-4 animate-spin" /> : <RefreshCw className="h-4 w-4" />}
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {isLoading && listings.length === 0 ? (
          <div className="flex items-center justify-center py-12" role="status" aria-label="Loading listings">
            <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        ) : listings.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 text-center">
            <p className="text-muted-foreground">No listings found</p>
          </div>
        ) : (
          <>
            <div className="overflow-x-auto">
              <table className="w-full" role="table">
                <thead>
                  <tr className="border-b text-left text-sm text-muted-foreground">
                    <th className="pb-3 font-medium">Platform</th>
                    <th className="pb-3 font-medium">Listing ID</th>
                    <th className="pb-3 font-medium">Product</th>
                    <th className="pb-3 font-medium">State</th>
                    <th className="pb-3 font-medium">Price</th>
                    <th className="pb-3 font-medium">Created</th>
                    <th className="pb-3 font-medium">Updated</th>
                    <th className="pb-3 font-medium">Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {listings.map((listing) => (
                    <tr key={listing.id} className="border-b hover:bg-muted/50 transition-colors">
                      <td className="py-3">
                        <span className="flex items-center gap-1">
                          {platformIcons[listing.platform]}
                          <span className="font-medium text-sm">{formatPlatform(listing.platform)}</span>
                        </span>
                      </td>
                      <td className="py-3 font-mono text-sm">{listing.platform_listing_id}</td>
                      <td className="py-3 text-sm text-muted-foreground max-w-xs truncate">
                        {listing.product_id.slice(0, 12)}...
                      </td>
                      <td className="py-3">
                        <Badge variant={stateVariants[listing.state] || 'outline'} className="gap-1">
                          {stateIcons[listing.state]}
                          {formatState(listing.state)}
                        </Badge>
                      </td>
                      <td className="py-3 text-sm font-medium">${listing.price}</td>
                      <td className="py-3 text-sm text-muted-foreground">
                        {new Date(listing.created_at).toLocaleDateString()}
                      </td>
                      <td className="py-3 text-sm text-muted-foreground">
                        {new Date(listing.updated_at).toLocaleDateString()}
                      </td>
                      <td className="py-3">
                        <div className="flex items-center gap-1">
                          <Button
                            variant="ghost"
                            size="icon"
                            onClick={() => handleReprice(listing)}
                            disabled={listing.state !== 'live'}
                            title="Reprice"
                          >
                            <DollarSign className="h-3 w-3" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="icon"
                            onClick={() => handleDelist(listing)}
                            disabled={listing.state !== 'live' && listing.state !== 'suspended'}
                            title="Delist"
                            className="text-destructive hover:text-destructive"
                          >
                            <Trash2 className="h-3 w-3" />
                          </Button>
                        </div>
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
                  Showing {query.offset + 1} to {Math.min(query.offset + listings.length, total)} of {total}
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
                    disabled={query.offset + listings.length >= total}
                  >
                    Next
                  </Button>
                </div>
              </div>
            )}

            {/* Reprice Dialog */}
            {selectedListing && showRepriceDialog && (
              <RepriceDialog
                listing={selectedListing}
                onClose={() => { setShowRepriceDialog(false); setSelectedListing(null); }}
                onSuccess={handleRepriceSuccess}
              />
            )}
          </>
        )}
      </CardContent>
    </Card>
  );
}

// Reprice Dialog Component
interface RepriceDialogProps {
  listing: ListingResponse;
  onClose: () => void;
  onSuccess?: () => void;
}

function RepriceDialog({ listing, onClose, onSuccess }: RepriceDialogProps) {
  const [isLoading, setIsLoading] = useState(false);
  const [isLoadingPreview, setIsLoadingPreview] = useState(false);
  const [feePreview, setFeePreview] = useState<FeePreview | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [basePrice, setBasePrice] = useState(listing.price);
  const [platform, _setPlatform] = useState(listing.platform);

  // Fetch fee preview when base price or platform changes
  const fetchFeePreview = async (basePriceStr: string, platformStr: Platform) => {
    if (!basePriceStr || parseFloat(basePriceStr) <= 0) {
      setFeePreview(null);
      return;
    }
    setIsLoadingPreview(true);
    try {
      const response: RepriceResponse = await api.listings.reprice(listing.product_id, {
        platform: platformStr,
        base_price: basePriceStr,
      });
      setFeePreview(response.fee_preview);
    } catch (err) {
      console.error('Failed to fetch fee preview:', err);
      setFeePreview(null);
    } finally {
      setIsLoadingPreview(false);
    }
  };

  // Debounce fee preview fetch
  useEffect(() => {
    const timer = setTimeout(() => {
      fetchFeePreview(basePrice, platform);
    }, 500);
    return () => clearTimeout(timer);
  }, [basePrice, platform]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);
    setError(null);
    try {
      const repriceRequest: RepriceRequest = {
        platform,
        base_price: basePrice,
      };
      await api.listings.reprice(listing.product_id, repriceRequest);
      onClose();
      onSuccess?.();
    } catch (err: any) {
      setError(err.message ?? 'Failed to reprice');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Dialog open={true} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="max-w-md max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <div className="flex items-start justify-between">
            <div>
              <DialogTitle>Reprice Listing</DialogTitle>
              <p className="text-sm text-muted-foreground mt-1">
                {listing.platform_listing_id} on {listing.platform}
              </p>
            </div>
            <Button variant="ghost" size="icon" onClick={onClose}>
              <X className="h-4 w-4" />
            </Button>
          </div>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="space-y-4 py-4">
          {error && (
            <div className="p-3 text-sm text-destructive bg-destructive/10 rounded-md" role="alert">
              {error}
            </div>
          )}

          {/* Platform (read-only for reprice) */}
          <div className="space-y-2">
            <label className="text-sm font-medium">Platform</label>
            <div className="flex items-center gap-2 px-3 py-2 border rounded-md bg-muted">
              <Badge variant="outline">{listing.platform}</Badge>
            </div>
          </div>

          {/* Base Price */}
          <div className="space-y-2">
            <label htmlFor="base_price" className="text-sm font-medium">
              Base Price (USD) <span className="text-destructive">*</span>
            </label>
            <div className="relative">
              <DollarSign className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground h-4 w-4" />
              <Input
                id="base_price"
                type="number"
                step="0.01"
                min="0.01"
                value={basePrice}
                onChange={(e) => setBasePrice(e.target.value)}
                placeholder="e.g., 29.99"
                className="pl-10"
              />
            </div>
            <p className="text-xs text-muted-foreground">Current price: ${listing.price}</p>
          </div>

          {/* Fee Preview */}
          {feePreview && (
            <Card variant="outline" className="bg-muted/50">
              <CardHeader className="pb-2">
                <CardTitle className="flex items-center gap-2 text-base">
                  <TrendingUp className="h-4 w-4" />
                  Fee Preview
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-2 pt-0">
                <div className="grid grid-cols-2 gap-2 text-sm">
                  <div className="space-y-1">
                    <p className="text-muted-foreground">Platform Fee %</p>
                    <p className="font-medium">{feePreview.platform_fee_percent}%</p>
                  </div>
                  <div className="space-y-1">
                    <p className="text-muted-foreground">Transaction Fee</p>
                    <p className="font-medium">${feePreview.transaction_fee}</p>
                  </div>
                </div>
                <div className="border-t pt-2 space-y-1">
                  <p className="text-xs text-muted-foreground">Net Prices by License Tier:</p>
                  <div className="grid grid-cols-3 gap-2 text-xs">
                    <div className="space-y-1 p-2 bg-background rounded">
                      <p className="text-muted-foreground">Personal</p>
                      <p className="font-medium">${feePreview.personal_net}</p>
                    </div>
                    <div className="space-y-1 p-2 bg-background rounded">
                      <p className="text-muted-foreground">Small Commercial</p>
                      <p className="font-medium">${feePreview.small_commercial_net}</p>
                    </div>
                    <div className="space-y-1 p-2 bg-background rounded">
                      <p className="text-muted-foreground">Extended</p>
                      <p className="font-medium">${feePreview.extended_commercial_net}</p>
                    </div>
                  </div>
                </div>
                <p className="text-xs text-muted-foreground">
                  Includes Payoneer ≤3% withdrawal fee per R9
                </p>
              </CardContent>
            </Card>
          )}

          {isLoadingPreview && !feePreview && (
            <div className="flex items-center justify-center py-4">
              <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
            </div>
          )}

          <DialogFooter className="gap-2">
            <Button type="button" variant="outline" onClick={onClose}>
              Cancel
            </Button>
            <Button type="submit" disabled={isLoading}>
              {isLoading ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Updating...
                </>
              ) : (
                <>
                  <DollarSign className="mr-2 h-4 w-4" />
                  Update Price
                </>
              )}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}