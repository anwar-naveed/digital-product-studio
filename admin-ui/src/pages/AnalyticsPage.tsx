// Analytics page component
import { useListingsAnalytics } from '../hooks/useAnalytics';
import { Card, CardContent } from '../components/ui/Card';
import { LoadingSpinner } from '../components/LoadingSpinner';
import { formatCurrency, formatNumber } from '../lib/utils';
import { TrendingUp, TrendingDown, DollarSign, Eye, ShoppingCart } from 'lucide-react';
import { useState } from 'react';

export function AnalyticsPage() {
  const [query, setQuery] = useState({ platform: '', limit: 20, offset: 0 });
  const { data, isLoading, error } = useListingsAnalytics(query);

  const platformOptions = [
    { value: '', label: 'All Platforms' },
    { value: 'etsy', label: 'Etsy' },
    { value: 'lemon-squeezy', label: 'Lemon Squeezy' },
    { value: 'printify', label: 'Printify' },
    { value: 'gelato', label: 'Gelato' },
  ];

  // Calculate summary stats
  const totalRevenue = data?.items.reduce((sum, a) => sum + parseFloat(a.total_revenue), 0) ?? 0;
  const totalFees = data?.items.reduce((sum, a) => sum + parseFloat(a.total_fees), 0) ?? 0;
  const totalNet = data?.items.reduce((sum, a) => sum + parseFloat(a.total_net), 0) ?? 0;
  const totalSales = data?.items.reduce((sum, a) => sum + a.sales_count, 0) ?? 0;
  const totalViews = data?.items.reduce((sum, a) => sum + a.views, 0) ?? 0;

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  if (error) {
    return <div className="text-red-500">Failed to load analytics</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white">Analytics</h1>
          <p className="text-gray-600 dark:text-gray-400 mt-1">Per-listing performance: views, sales, and margin</p>
        </div>
        <select
          value={query.platform}
          onChange={(e) => setQuery({ ...query, platform: e.target.value })}
          className="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700"
        >
          {platformOptions.map((opt) => (
            <option key={opt.value} value={opt.value}>{opt.label}</option>
          ))}
        </select>
      </div>

      {/* Summary Cards */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-5">
        <Card>
          <CardContent className="flex items-center gap-4 p-6">
            <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-blue-500/10">
              <Eye className="h-6 w-6 text-blue-500" />
            </div>
            <div>
              <p className="text-sm text-gray-500 dark:text-gray-400">Total Views</p>
              <p className="text-2xl font-bold text-gray-900 dark:text-white">{formatNumber(totalViews)}</p>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="flex items-center gap-4 p-6">
            <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-green-500/10">
              <ShoppingCart className="h-6 w-6 text-green-500" />
            </div>
            <div>
              <p className="text-sm text-gray-500 dark:text-gray-400">Total Sales</p>
              <p className="text-2xl font-bold text-gray-900 dark:text-white">{formatNumber(totalSales)}</p>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="flex items-center gap-4 p-6">
            <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-purple-500/10">
              <DollarSign className="h-6 w-6 text-purple-500" />
            </div>
            <div>
              <p className="text-sm text-gray-500 dark:text-gray-400">Total Revenue</p>
              <p className="text-2xl font-bold text-gray-900 dark:text-white">{formatCurrency(totalRevenue)}</p>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="flex items-center gap-4 p-6">
            <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-orange-500/10">
              <TrendingDown className="h-6 w-6 text-orange-500" />
            </div>
            <div>
              <p className="text-sm text-gray-500 dark:text-gray-400">Total Fees</p>
              <p className="text-2xl font-bold text-gray-900 dark:text-white">{formatCurrency(totalFees)}</p>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="flex items-center gap-4 p-6">
            <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-green-500/10">
              <TrendingUp className="h-6 w-6 text-green-500" />
            </div>
            <div>
              <p className="text-sm text-gray-500 dark:text-gray-400">Net Profit</p>
              <p className="text-2xl font-bold text-gray-900 dark:text-white">{formatCurrency(totalNet)}</p>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Analytics Table */}
      <Card>
        <CardContent className="p-0">
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b bg-gray-50 dark:bg-gray-800/50">
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Product</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Platform</th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">State</th>
                  <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Views</th>
                  <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Favorites</th>
                  <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Sales</th>
                  <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Revenue</th>
                  <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Fees</th>
                  <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Net</th>
                  <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Margin</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-200 dark:divide-gray-700">
                {data?.items?.map((item) => (
                  <tr key={item.listing_id} className="hover:bg-gray-50 dark:hover:bg-gray-800/50">
                    <td className="px-6 py-4 font-medium text-gray-900 dark:text-white">{item.product_title}</td>
                    <td className="px-6 py-4 capitalize text-gray-600 dark:text-gray-300">
                      {item.platform.replace('-', ' ')}
                    </td>
                    <td className="px-6 py-4">
                      <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300">
                        {item.state}
                      </span>
                    </td>
                    <td className="px-6 py-4 text-right text-gray-600 dark:text-gray-300">{formatNumber(item.views)}</td>
                    <td className="px-6 py-4 text-right text-gray-600 dark:text-gray-300">{formatNumber(item.favorites)}</td>
                    <td className="px-6 py-4 text-right font-medium text-gray-900 dark:text-white">{formatNumber(item.sales_count)}</td>
                    <td className="px-6 py-4 text-right text-gray-900 dark:text-white">{formatCurrency(item.total_revenue)}</td>
                    <td className="px-6 py-4 text-right text-gray-600 dark:text-gray-300">{formatCurrency(item.total_fees)}</td>
                    <td className="px-6 py-4 text-right font-medium text-green-600 dark:text-green-400">{formatCurrency(item.total_net)}</td>
                    <td className="px-6 py-4 text-right text-gray-600 dark:text-gray-300">{parseFloat(item.margin_percent).toFixed(1)}%</td>
                  </tr>
                ))}
                {(!data?.items.length) && (
                  <tr>
                    <td colSpan={10} className="px-6 py-12 text-center text-gray-500 dark:text-gray-400">
                      No analytics data available yet.
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}