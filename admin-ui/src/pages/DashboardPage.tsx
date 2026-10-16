// Dashboard page component
import { useNiches } from '../hooks/useNiches';
import { useRuns } from '../hooks/useRuns';
import { useProducts } from '../hooks/useProducts';
import { useListings } from '../hooks/useListings';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/Card';
import { LoadingSpinner } from '../components/LoadingSpinner';
import { formatNumber } from '../lib/utils';
import { Package, PlayCircle, ClipboardList, Store, TrendingUp } from 'lucide-react';

export function DashboardPage() {
  const { data: nichesData, isLoading: nichesLoading } = useNiches({ limit: 1, offset: 0 });
  const { data: runsData, isLoading: runsLoading } = useRuns({ limit: 1, offset: 0 });
  const { data: productsData, isLoading: productsLoading } = useProducts({ limit: 1, offset: 0 });
  const { data: listingsData, isLoading: listingsLoading } = useListings({ limit: 1, offset: 0 });

  const isLoading = nichesLoading || runsLoading || productsLoading || listingsLoading;

  const stats = [
    {
      label: 'Total Niches',
      value: nichesData?.total ?? 0,
      icon: Package,
      color: 'text-blue-500',
      bgColor: 'bg-blue-500/10',
    },
    {
      label: 'Active Runs',
      value: runsData?.items?.filter((r) => r.status === 'running' || r.status === 'pending').length ?? 0,
      icon: PlayCircle,
      color: 'text-green-500',
      bgColor: 'bg-green-500/10',
    },
    {
      label: 'Products',
      value: productsData?.total ?? 0,
      icon: Store,
      color: 'text-purple-500',
      bgColor: 'bg-purple-500/10',
    },
    {
      label: 'Listings',
      value: listingsData?.total ?? 0,
      icon: ClipboardList,
      color: 'text-orange-500',
      bgColor: 'bg-orange-500/10',
    },
  ];

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Dashboard</h1>
        <p className="text-muted-foreground mt-1">
          Overview of your digital product studio pipeline
        </p>
      </div>

      {/* Stats Grid */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {stats.map((stat) => (
          <Card key={stat.label}>
            <CardContent className="flex items-center gap-4 p-6">
              <div className={`flex h-12 w-12 items-center justify-center rounded-xl ${stat.bgColor}`}>
                <stat.icon className={`h-6 w-6 ${stat.color}`} />
              </div>
              <div>
                <p className="text-sm text-muted-foreground">{stat.label}</p>
                <p className="text-2xl font-bold text-foreground">
                  {formatNumber(stat.value)}
                </p>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Quick Actions */}
      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Quick Actions</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <button className="w-full flex items-center justify-between px-4 py-3 text-left border rounded-lg hover:bg-accent transition-colors">
              <span>Create New Niche</span>
              <TrendingUp className="h-4 w-4 text-gray-400" />
            </button>
            <button className="w-full flex items-center justify-between px-4 py-3 text-left border rounded-lg hover:bg-accent transition-colors">
              <span>Spawn Production Run</span>
              <PlayCircle className="h-4 w-4 text-gray-400" />
            </button>
            <button className="w-full flex items-center justify-between px-4 py-3 text-left border rounded-lg hover:bg-accent transition-colors">
              <span>Review Curation Queue</span>
              <ClipboardList className="h-4 w-4 text-gray-400" />
            </button>
          </CardContent>
        </Card>

        <Card className="md:col-span-2">
          <CardHeader>
            <CardTitle className="text-lg">Recent Activity</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="flex items-center gap-4 p-3 bg-muted rounded-lg">
                <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-green-500/10">
                  <PlayCircle className="h-5 w-5 text-green-500" />
                </div>
                <div className="flex-1">
                  <p className="font-medium text-foreground">Production run completed</p>
                  <p className="text-sm text-muted-foreground">2 minutes ago</p>
                </div>
              </div>
              <div className="flex items-center gap-4 p-3 bg-muted rounded-lg">
                <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-blue-500/10">
                  <Package className="h-5 w-5 text-blue-500" />
                </div>
                <div className="flex-1">
                  <p className="font-medium text-foreground">New niche created: Gothic Wall Art</p>
                  <p className="text-sm text-muted-foreground">1 hour ago</p>
                </div>
              </div>
              <div className="flex items-center gap-4 p-3 bg-muted rounded-lg">
                <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-purple-500/10">
                  <Store className="h-5 w-5 text-purple-500" />
                </div>
                <div className="flex-1">
                  <p className="font-medium text-foreground">Product published to Etsy</p>
                  <p className="text-sm text-muted-foreground">3 hours ago</p>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}