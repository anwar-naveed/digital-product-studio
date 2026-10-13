'use client';

import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { Loader2, X, CheckCircle, AlertCircle, Globe, Zap, Printer, Box } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/Button';
import { Card, CardContent } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/Select';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter, DialogTrigger } from '@/components/ui/Dialog';

import { api, type Platform, type LicenseTier, type CredentialMetadata, type PublishRequest, type PublishResponse } from '@/api/client';

const PublishSchema = z.object({
  platform: z.enum(['etsy', 'lemon-squeezy', 'printify', 'gelato']),
  license_tier: z.enum(['personal', 'small_commercial', 'extended_commercial']).optional(),
});

type PublishFormData = z.infer<typeof PublishSchema>;

const platformIcons: Record<Platform, React.ReactNode> = {
  etsy: <Globe className="h-4 w-4" />,
  'lemon-squeezy': <Zap className="h-4 w-4" />,
  printify: <Printer className="h-4 w-4" />,
  gelato: <Box className="h-4 w-4" />,
};

const platformLabels: Record<Platform, string> = {
  etsy: 'Etsy',
  'lemon-squeezy': 'Lemon Squeezy',
  printify: 'Printify',
  gelato: 'Gelato',
};

interface PublishDialogProps {
  productId: string;
  productTitle: string;
  onSuccess?: (response: PublishResponse) => void;
}

export function PublishDialog({ productId, productTitle, onSuccess }: PublishDialogProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [credentials, setCredentials] = useState<CredentialMetadata[]>([]);
  const [_isLoadingCredentials, setIsLoadingCredentials] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const {
    register,
    handleSubmit,
    watch,
    setValue,
    formState: { errors: _errors, isSubmitting },
    reset,
  } = useForm<PublishFormData>({
    resolver: zodResolver(PublishSchema),
    defaultValues: {
      platform: 'etsy',
      license_tier: 'personal',
    },
  });

  const loadCredentials = async () => {
    setIsLoadingCredentials(true);
    try {
      const data = await api.settings.credentials();
      setCredentials(data);
    } catch (err) {
      console.error('Failed to load credentials:', err);
    } finally {
      setIsLoadingCredentials(false);
    }
  };

  const watchedPlatform = watch('platform');

  const onSubmit = async (data: PublishFormData) => {
    setIsLoading(true);
    setError(null);
    try {
      const publishRequest: PublishRequest = {
        platform: data.platform as Platform,
        license_tier: data.license_tier as LicenseTier,
      };
      const response = await api.listings.publish(productId, publishRequest);
      reset();
      setIsOpen(false);
      onSuccess?.(response);
    } catch (err: any) {
      setError(err.message ?? 'Failed to publish');
    } finally {
      setIsLoading(false);
    }
  };

  const hasCredentials = credentials.find(c => c.platform === watchedPlatform)?.has_credentials ?? false;

  return (
    <>
      <DialogTrigger asChild>
        <Button onClick={() => { loadCredentials(); setIsOpen(true); }}>
          <CheckCircle className="mr-2 h-4 w-4" />
          Publish
        </Button>
      </DialogTrigger>

      <Dialog open={isOpen} onOpenChange={setIsOpen}>
        <DialogContent className="max-w-md max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <div className="flex items-start justify-between">
              <div>
                <DialogTitle>Publish Product</DialogTitle>
                <p className="text-sm text-muted-foreground mt-1">{productTitle}</p>
              </div>
              <Button variant="ghost" size="icon" onClick={() => { reset(); setIsOpen(false); }}>
                <X className="h-4 w-4" />
              </Button>
            </div>
          </DialogHeader>
          <form onSubmit={handleSubmit(onSubmit)} className="space-y-4 py-4">
            {error && (
              <div className="p-3 text-sm text-destructive bg-destructive/10 rounded-md" role="alert">
                {error}
              </div>
            )}

            {/* Platform Selection */}
            <div className="space-y-2">
              <label className="text-sm font-medium">
                Platform <span className="text-destructive">*</span>
              </label>
              <div className="grid gap-2 sm:grid-cols-2">
                {(['etsy', 'lemon-squeezy', 'printify', 'gelato'] as Platform[]).map((platform) => (
                  <label
                    key={platform}
                    className={cn(
                      'relative flex cursor-pointer items-center justify-center rounded-lg border p-4 transition-all',
                      'hover:border-primary/50',
                      'data-[state=checked]:border-primary data-[state=checked]:bg-primary/5',
                      watchedPlatform === platform && 'border-primary bg-primary/5'
                    )}
                  >
                    <input
                      type="radio"
                      {...register('platform')}
                      value={platform}
                      className="sr-only"
                    />
                    <div className="flex flex-col items-center gap-2">
                      <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-muted">
                        {platformIcons[platform]}
                      </div>
                      <span className="text-sm font-medium">{platformLabels[platform]}</span>
                      {!hasCredentials && platform === watchedPlatform && (
                        <Badge variant="destructive" className="text-xs">
                          No credentials
                        </Badge>
                      )}
                    </div>
                  </label>
                ))}
              </div>
            </div>

            {/* License Tier */}
            <div className="space-y-2">
              <label htmlFor="license_tier" className="text-sm font-medium">
                License Tier
              </label>
              <Select
                value={watch('license_tier')}
                onValueChange={(value) => setValue('license_tier', value as LicenseTier)}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select license tier" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="personal">Personal Use</SelectItem>
                  <SelectItem value="small_commercial">Small Commercial (up to 500 units)</SelectItem>
                  <SelectItem value="extended_commercial">Extended Commercial (unlimited)</SelectItem>
                </SelectContent>
              </Select>
            </div>

            {/* Credentials Status */}
            {watchedPlatform && (
              <Card variant="outline" className={cn(
                'bg-muted/50',
                hasCredentials ? 'border-green-200 dark:border-green-800' : 'border-destructive/50'
              )}>
                <CardContent className="pt-4">
                  <div className="flex items-center gap-2">
                    {hasCredentials ? (
                      <>
                        <CheckCircle className="h-4 w-4 text-green-500" />
                        <span className="text-sm text-green-700 dark:text-green-400">
                          Credentials configured for {platformLabels[watchedPlatform]}
                        </span>
                      </>
                    ) : (
                      <>
                        <AlertCircle className="h-4 w-4 text-destructive" />
                        <span className="text-sm text-destructive">
                          No credentials configured for {platformLabels[watchedPlatform]}
                        </span>
                      </>
                    )}
                    <Button
                      variant="ghost"
                      size="sm"
                      className="ml-auto"
                      onClick={() => window.location.href = '/settings/credentials'}
                    >
                      Manage Credentials
                    </Button>
                  </div>
                </CardContent>
              </Card>
            )}

            <DialogFooter className="gap-2">
              <Button type="button" variant="outline" onClick={() => { reset(); setIsOpen(false); }}>
                Cancel
              </Button>
              <Button type="submit" disabled={isLoading || isSubmitting || !hasCredentials}>
                {isLoading || isSubmitting ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    Publishing...
                  </>
                ) : (
                  <>
                    <CheckCircle className="mr-2 h-4 w-4" />
                    Publish to {platformLabels[watchedPlatform as Platform] || 'Platform'}
                  </>
                )}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </>
  );
}