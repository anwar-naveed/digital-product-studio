// Settings page component
import { useCredentials, useUpdateCredential, useFeeTables, useUpdateFeeTable } from '../hooks/useSettings';
import { credentialFields } from '../components/SettingsPages';
import type { Platform } from '../api/client';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/Card';
import { LoadingSpinner } from '../components/LoadingSpinner';
import { formatDateTime } from '../lib/utils';
import { Key, Shield, DollarSign, Save } from 'lucide-react';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';

const PLATFORMS = [
  { value: 'etsy', label: 'Etsy', icon: '🏪' },
  { value: 'lemon-squeezy', label: 'Lemon Squeezy', icon: '🍋' },
  { value: 'printify', label: 'Printify', icon: '🖨️' },
  { value: 'gelato', label: 'Gelato', icon: '🎨' },
] as const;

const feeTableSchema = z.object({
  platform_fee_percent: z.string().min(1, 'Required'),
  transaction_fee: z.string().min(1, 'Required'),
  payoneer_fee_percent: z.string().optional(),
  currency: z.string().optional(),
});

type FeeTableFormData = z.infer<typeof feeTableSchema>;

export function SettingsPage() {
  const [activeTab, setActiveTab] = useState<'credentials' | 'fee-tables'>('credentials');
  const [editingPlatform, setEditingPlatform] = useState<string | null>(null);
  const [feeFormData, setFeeFormData] = useState<Record<string, FeeTableFormData>>({});
  
  const { data: credentials, isLoading: credsLoading } = useCredentials();
  const { data: feeTables, isLoading: feeLoading } = useFeeTables();
  const updateCredential = useUpdateCredential();
  const updateFeeTable = useUpdateFeeTable();

  const isLoading = credsLoading || feeLoading;

  const handleSaveCredential = async (platform: string, fields: Record<string, string>) => {
    await updateCredential.mutateAsync({ platform: platform as any, fields });
    setEditingPlatform(null);
  };

  const handleSaveFeeTable = async (platform: string) => {
    const data = feeFormData[platform];
    if (!data) return;
    await updateFeeTable.mutateAsync({ platform: platform as any, data });
    setFeeFormData(prev => ({ ...prev, [platform]: data }));
  };

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
        <h1 className="text-3xl font-bold text-foreground">Settings</h1>
        <p className="text-muted-foreground mt-1">Configure credentials and platform fee tables</p>
      </div>

      {/* Tabs */}
      <div className="flex border-b">
        <button
          onClick={() => setActiveTab('credentials')}
          className={`px-6 py-3 text-sm font-medium border-b-2 transition-colors ${
            activeTab === 'credentials'
              ? 'border-primary text-primary'
              : 'border-transparent text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'
          }`}
        >
          <Key className="h-4 w-4 mr-2 inline" /> Credentials
        </button>
        <button
          onClick={() => setActiveTab('fee-tables')}
          className={`px-6 py-3 text-sm font-medium border-b-2 transition-colors ${
            activeTab === 'fee-tables'
              ? 'border-primary text-primary'
              : 'border-transparent text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'
          }`}
        >
          <DollarSign className="h-4 w-4 mr-2 inline" /> Fee Tables
        </button>
      </div>

      {activeTab === 'credentials' && (
        <div className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Key className="h-5 w-5" /> Platform Credentials
              </CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-sm text-muted-foreground mb-4">
                Credentials are stored securely in Openship as secrets. Only reference metadata is shown here.
              </p>
              <div className="space-y-4">
                {PLATFORMS.map((platform) => {
                  const cred = credentials?.find((c) => c.platform === platform.value);
                  const isEditing = editingPlatform === platform.value;
                  
                  return (
                    <div key={platform.value} className="flex items-center justify-between p-4 border rounded-lg bg-muted">
                      <div className="flex items-center gap-4">
                        <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
                          <span className="text-xl">{platform.icon}</span>
                        </div>
                        <div>
                          <p className="font-medium text-foreground">{platform.label}</p>
                          <p className="text-sm text-muted-foreground">
                            {cred?.has_credentials ? (
                              <>
                                <Shield className="h-3 w-3 mr-1 inline text-green-500" />
                                Credentials configured
                                {cred?.last_validated && (
                                  <span className="ml-2">• Last validated: {formatDateTime(cred.last_validated)}</span>
                                )}
                              </>
                            ) : (
                              'No credentials configured'
                            )}
                          </p>
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        {isEditing ? (
                          <CredentialEditForm
                            platform={platform.value}
                            onSave={handleSaveCredential}
                            onCancel={() => setEditingPlatform(null)}
                          />
                        ) : (
                          <button
                            onClick={() => setEditingPlatform(platform.value)}
                            className="px-3 py-1.5 text-sm bg-primary text-primary-foreground rounded hover:bg-primary/90"
                          >
                            {cred?.has_credentials ? 'Update' : 'Configure'}
                          </button>
                        )}
                      </div>
                    </div>
                  );
                })}
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {activeTab === 'fee-tables' && (
        <div className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <DollarSign className="h-5 w-5" /> Platform Fee Tables
              </CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-sm text-muted-foreground mb-4">
                Configure platform fees for accurate pricing and margin calculations. Includes Payoneer withdrawal fees for Etsy.
              </p>
              <div className="space-y-4">
                {PLATFORMS.map((platform) => {
                  const feeTable = feeTables?.find((f) => f.platform === platform.value);
                  const rules = feeTable?.rules as Record<string, unknown> | undefined;
                  
                  return (
                    <div key={platform.value} className="p-4 border rounded-lg bg-muted">
                      <div className="flex items-center justify-between mb-4">
                        <div className="flex items-center gap-3">
                          <span className="text-xl">{platform.icon}</span>
                          <h4 className="font-medium text-foreground">{platform.label}</h4>
                        </div>
                        {feeTable && (
                          <span className="text-xs text-gray-500">
                            Updated: {formatDateTime(feeTable.effective_from)}
                          </span>
                        )}
                      </div>
                      <FeeTableForm
                        platform={platform.value}
                        initialData={{
                          platform_fee_percent: (rules?.platform_fee_percent as string) ?? '',
                          transaction_fee: (rules?.transaction_fee as string) ?? '',
                          payoneer_fee_percent: (rules?.payoneer_fee_percent as string) ?? '',
                          currency: (rules?.currency as string) ?? 'USD',
                        }}
                        onSave={() => handleSaveFeeTable(platform.value)}
                      />
                    </div>
                  );
                })}
              </div>
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  );
}

function CredentialEditForm({ platform, onSave, onCancel }: { platform: string; onSave: (platform: string, fields: Record<string, string>) => void; onCancel: () => void }) {
  const [values, setValues] = useState<Record<string, string>>({});
  const fields = credentialFields[platform as Platform] ?? [];

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        const payload: Record<string, string> = {};
        for (const [k, v] of Object.entries(values)) {
          if (v.trim()) payload[k] = v.trim();
        }
        onSave(platform, payload);
      }}
      className="space-y-2 w-full"
    >
      {fields.map((field) => (
        <div key={field.key} className="flex items-center gap-2">
          <label className="w-40 shrink-0 text-xs font-medium text-foreground" title={field.hint}>
            {field.label} {field.required && <span className="text-destructive">*</span>}
          </label>
          <input
            type={field.type === 'password' ? 'password' : 'text'}
            value={values[field.key] ?? ''}
            onChange={(e) => setValues((prev) => ({ ...prev, [field.key]: e.target.value }))}
            placeholder={field.placeholder ?? ''}
            autoComplete="off"
            className="flex-1 px-3 py-1.5 text-sm border border-input rounded bg-background text-foreground"
          />
        </div>
      ))}
      <div className="flex items-center gap-2 justify-end">
        <button type="submit" className="px-3 py-1.5 text-sm bg-primary text-primary-foreground rounded hover:bg-primary/90">
          <Save className="h-3 w-3 mr-1 inline" /> Save
        </button>
        <button type="button" onClick={onCancel} className="px-3 py-1.5 text-sm border border-border rounded hover:bg-accent text-foreground">
          Cancel
        </button>
      </div>
    </form>
  );
}

function FeeTableForm({ platform: _platform, initialData, onSave }: { platform: string; initialData: FeeTableFormData; onSave: (data: FeeTableFormData) => void }) {
  const { register, handleSubmit, formState: { errors } } = useForm<FeeTableFormData>({
    resolver: zodResolver(feeTableSchema),
    defaultValues: initialData,
  });

  return (
    <form onSubmit={handleSubmit(onSave)} className="grid gap-4 md:grid-cols-4">
      <div>
        <label className="block text-sm font-medium mb-1">Platform Fee %</label>
        <input
          {...register('platform_fee_percent')}
          type="number"
          step="0.01"
          className="w-full px-3 py-1.5 text-sm border border-border rounded bg-white dark:bg-gray-700"
        />
        {errors.platform_fee_percent && <p className="text-destructive text-xs mt-1">{errors.platform_fee_percent.message}</p>}
      </div>
      <div>
        <label className="block text-sm font-medium mb-1">Transaction Fee</label>
        <input
          {...register('transaction_fee')}
          type="number"
          step="0.01"
          className="w-full px-3 py-1.5 text-sm border border-border rounded bg-white dark:bg-gray-700"
        />
        {errors.transaction_fee && <p className="text-destructive text-xs mt-1">{errors.transaction_fee.message}</p>}
      </div>
      <div>
        <label className="block text-sm font-medium mb-1">Payoneer Fee %</label>
        <input
          {...register('payoneer_fee_percent')}
          type="number"
          step="0.01"
          className="w-full px-3 py-1.5 text-sm border border-border rounded bg-white dark:bg-gray-700"
        />
      </div>
      <div>
        <label className="block text-sm font-medium mb-1">Currency</label>
        <input
          {...register('currency')}
          className="w-full px-3 py-1.5 text-sm border border-border rounded bg-white dark:bg-gray-700"
        />
      </div>
      <div className="md:col-span-4 flex justify-end">
        <button type="submit" className="px-4 py-1.5 text-sm bg-primary text-primary-foreground rounded hover:bg-primary/90">
          <Save className="h-3 w-3 mr-1 inline" /> Save Fee Table
        </button>
      </div>
    </form>
  );
}