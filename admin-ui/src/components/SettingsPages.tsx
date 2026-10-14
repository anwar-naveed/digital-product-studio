'use client';

import { useState, useCallback, useEffect } from 'react';
import { Loader2, Key, Shield, DollarSign, Globe, Zap, Printer, Box, Save, AlertTriangle } from 'lucide-react';

import { Button } from '@/components/ui/Button';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';
import { Input } from '@/components/ui/Input';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/Tabs';
import { Textarea } from '@/components/ui/Textarea';
import { DialogDescription, Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/Dialog';
import { api, type CredentialMetadata, type FeeTableResponse, type Platform } from '@/api/client';


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

export function SettingsPages() {
  const [activeTab, setActiveTab] = useState<'credentials' | 'fee-tables' | 'webhooks'>('credentials');
  const [credentials, setCredentials] = useState<CredentialMetadata[]>([]);
  const [feeTables, setFeeTables] = useState<FeeTableResponse[]>([]);
  const [isLoadingCredentials, setIsLoadingCredentials] = useState(true);
  const [isLoadingFeeTables, setIsLoadingFeeTables] = useState(true);
  const [showCredentialDialog, setShowCredentialDialog] = useState<Platform | null>(null);
  const [credentialRef, setCredentialRef] = useState('');
  const [isUpdatingCredential, setIsUpdatingCredential] = useState(false);
  const [credentialError, setCredentialError] = useState<string | null>(null);
  const [showFeeTableDialog, setShowFeeTableDialog] = useState<Platform | null>(null);
  const [feeTableRules, setFeeTableRules] = useState('');
  const [isUpdatingFeeTable, setIsUpdatingFeeTable] = useState(false);
  const [feeTableError, setFeeTableError] = useState<string | null>(null);

  const loadCredentials = useCallback(async () => {
    setIsLoadingCredentials(true);
    try {
      const data = await api.settings.credentials();
      setCredentials(data.credentials);
    } catch (err) {
      console.error('Failed to load credentials:', err);
    } finally {
      setIsLoadingCredentials(false);
    }
  }, []);

  const loadFeeTables = useCallback(async () => {
    setIsLoadingFeeTables(true);
    try {
      const data = await api.settings.feeTables();
      setFeeTables(data.fee_tables);
    } catch (err) {
      console.error('Failed to load fee tables:', err);
    } finally {
      setIsLoadingFeeTables(false);
    }
  }, []);

  useEffect(() => {
    loadCredentials();
    loadFeeTables();
  }, [loadCredentials, loadFeeTables]);

  const handleUpdateCredential = async (platform: Platform) => {
    setIsUpdatingCredential(true);
    setCredentialError(null);
    try {
      await api.settings.updateCredential(platform, credentialRef);
      loadCredentials();
      setShowCredentialDialog(null);
      setCredentialRef('');
    } catch (err: any) {
      setCredentialError(err.message ?? 'Failed to update credential');
    } finally {
      setIsUpdatingCredential(false);
    }
  };

  const handleUpdateFeeTable = async (platform: Platform) => {
    setIsUpdatingFeeTable(true);
    setFeeTableError(null);
    try {
      const rules = JSON.parse(feeTableRules);
      await api.settings.updateFeeTable(platform, { rules });
      loadFeeTables();
      setShowFeeTableDialog(null);
      setFeeTableRules('');
    } catch (err: any) {
      if (err instanceof SyntaxError) {
        setFeeTableError('Invalid JSON format');
      } else {
        setFeeTableError(err.message ?? 'Failed to update fee table');
      }
    } finally {
      setIsUpdatingFeeTable(false);
    }
  };

  const openCredentialDialog = (platform: Platform) => {
    const cred = credentials.find(c => c.platform === platform);
    setCredentialRef(cred ? 'op://...' : '');
    setShowCredentialDialog(platform);
  };

  const openFeeTableDialog = (platform: Platform) => {
    const table = feeTables.find(t => t.platform === platform);
    setFeeTableRules(table ? JSON.stringify(table.rules, null, 2) : '{}');
    setShowFeeTableDialog(platform);
  };

  return (
    <Card className="w-full">
      <CardHeader>
        <div className="flex items-center gap-2">
          <Shield className="h-5 w-5" />
          <CardTitle>Settings</CardTitle>
        </div>
      </CardHeader>
      <CardContent>
        <Tabs value={activeTab} onValueChange={setActiveTab}>
          <TabsList className="grid w-full grid-cols-3">
            <TabsTrigger value="credentials">
              <Key className="mr-2 h-4 w-4" />
              Credentials
            </TabsTrigger>
            <TabsTrigger value="fee-tables">
              <DollarSign className="mr-2 h-4 w-4" />
              Fee Tables
            </TabsTrigger>
            <TabsTrigger value="webhooks">
              <AlertTriangle className="mr-2 h-4 w-4" />
              Webhooks
            </TabsTrigger>
          </TabsList>

          {/* Credentials Tab */}
          <TabsContent value="credentials" className="space-y-6">
            <div className="space-y-4">
              <p className="text-sm text-muted-foreground">
                Manage platform credential references. Secrets are stored securely in 1Password (op:// format) and never displayed here.
              </p>

              {isLoadingCredentials ? (
                <div className="flex items-center justify-center py-12">
                  <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                </div>
              ) : (
                <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                  {(['etsy', 'lemon-squeezy', 'printify', 'gelato'] as Platform[]).map((platform) => {
                    const cred = credentials.find(c => c.platform === platform);
                    const hasCreds = cred?.has_credentials ?? false;
                    return (
                      <Card key={platform} className="hover:shadow-md transition-shadow">
                        <CardHeader className="pb-3">
                          <div className="flex items-center justify-between">
                            <div className="flex items-center gap-2">
                              <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-muted">
                                {platformIcons[platform]}
                              </div>
                              <div>
                                <CardTitle className="text-base">{platformLabels[platform]}</CardTitle>
                                <CardDescription className="text-xs">
                                  {hasCreds ? 'Configured' : 'Not configured'}
                                </CardDescription>
                              </div>
                            </div>
                            <Badge variant={hasCreds ? 'success' : 'destructive'}>
                              {hasCreds ? 'Active' : 'Missing'}
                            </Badge>
                          </div>
                        </CardHeader>
                        <CardContent className="space-y-3 pt-0">
                          {hasCreds && cred && (
                            <div className="space-y-1 text-sm">
                              <p className="text-muted-foreground">Last validated:</p>
                              <p className="font-mono">
                                {cred.last_validated ? new Date(cred.last_validated).toLocaleString() : 'Never'}
                              </p>
                            </div>
                          )}
                          <Button
                            variant={hasCreds ? 'outline' : 'default'}
                            className="w-full"
                            onClick={() => openCredentialDialog(platform)}
                            disabled={isUpdatingCredential}
                          >
                            {hasCreds ? 'Update Reference' : 'Add Reference'}
                          </Button>
                        </CardContent>
                      </Card>
                    );
                  })}
                </div>
              )}

              {/* Credential Dialog */}
              {showCredentialDialog && (
                <Dialog open={true} onOpenChange={(open) => !open && setShowCredentialDialog(null)}>
                  <DialogContent className="max-w-md">
                    <DialogHeader>
                      <DialogTitle>
                        {platformLabels[showCredentialDialog]} Credential Reference
                      </DialogTitle>
                      <DialogDescription>
                        Enter the 1Password reference (op://vault/item/field) for this platform's credentials.
                        The actual secret values are never stored in this application.
                      </DialogDescription>
                    </DialogHeader>
                    <div className="space-y-4 py-4">
                      {credentialError && (
                        <div className="p-3 text-sm text-destructive bg-destructive/10 rounded-md" role="alert">
                          {credentialError}
                        </div>
                      )}
                      <div className="space-y-2">
                        <label htmlFor="credential_ref" className="text-sm font-medium">
                          1Password Reference <span className="text-destructive">*</span>
                        </label>
                        <Input
                          id="credential_ref"
                          value={credentialRef}
                          onChange={(e) => setCredentialRef(e.target.value)}
                          placeholder="op://vault/item/field"
                        />
                      </div>
                      <p className="text-xs text-muted-foreground">
                        Format: op://vault-name/item-name/field-name
                        The secret value is fetched at runtime and never persisted here.
                      </p>
                    </div>
                    <DialogFooter className="gap-2">
                      <Button variant="outline" onClick={() => { setShowCredentialDialog(null); setCredentialRef(''); }}>
                        Cancel
                      </Button>
                      <Button onClick={() => handleUpdateCredential(showCredentialDialog!)} disabled={isUpdatingCredential}>
                        {isUpdatingCredential ? (
                          <>
                            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                            Saving...
                          </>
                        ) : (
                          <>
                            <Save className="mr-2 h-4 w-4" />
                            Save Reference
                          </>
                        )}
                      </Button>
                    </DialogFooter>
                  </DialogContent>
                </Dialog>
              )}
            </div>
          </TabsContent>

          {/* Fee Tables Tab */}
          <TabsContent value="fee-tables" className="space-y-6">
            <div className="space-y-4">
              <p className="text-sm text-muted-foreground">
                Configure per-platform fee rules for pricing calculations. Rules are stored as JSON and used for reprice previews.
              </p>

              {isLoadingFeeTables ? (
                <div className="flex items-center justify-center py-12">
                  <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                </div>
              ) : (
                <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                  {(['etsy', 'lemon-squeezy', 'printify', 'gelato'] as Platform[]).map((platform) => {
                    const table = feeTables.find(t => t.platform === platform);
                    return (
                      <Card key={platform} className="hover:shadow-md transition-shadow">
                        <CardHeader className="pb-3">
                          <div className="flex items-center justify-between">
                            <div className="flex items-center gap-2">
                              <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-muted">
                                {platformIcons[platform]}
                              </div>
                              <div>
                                <CardTitle className="text-base">{platformLabels[platform]}</CardTitle>
                                <CardDescription className="text-xs">
                                  {table ? 'Configured' : 'Default rules'}
                                </CardDescription>
                              </div>
                            </div>
                            <Badge variant={table ? 'success' : 'outline'}>
                              {table ? 'Custom' : 'Default'}
                            </Badge>
                          </div>
                        </CardHeader>
                        <CardContent className="space-y-3 pt-0">
                          {table && (
                            <div className="space-y-1 text-sm">
                              <p className="text-muted-foreground">Effective from:</p>
                              <p className="font-mono">{new Date(table.effective_from).toLocaleDateString()}</p>
                            </div>
                          )}
                          <Button
                            variant={table ? 'outline' : 'default'}
                            className="w-full"
                            onClick={() => openFeeTableDialog(platform)}
                            disabled={isUpdatingFeeTable}
                          >
                            {table ? 'Edit Rules' : 'Configure Rules'}
                          </Button>
                        </CardContent>
                      </Card>
                    );
                  })}
                </div>
              )}

              {/* Fee Table Dialog */}
              {showFeeTableDialog && (
                <Dialog open={true} onOpenChange={(open) => !open && setShowFeeTableDialog(null)}>
                  <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
                    <DialogHeader>
                      <DialogTitle>
                        {platformLabels[showFeeTableDialog!]} Fee Rules
                      </DialogTitle>
                      <DialogDescription>
                        JSON configuration for platform fees. Includes platform fee %, transaction fees, and Payoneer withdrawal (≤3% per R9).
                      </DialogDescription>
                    </DialogHeader>
                    <div className="space-y-4 py-4">
                      {feeTableError && (
                        <div className="p-3 text-sm text-destructive bg-destructive/10 rounded-md" role="alert">
                          {feeTableError}
                        </div>
                      )}
                      <div className="space-y-2">
                        <label htmlFor="fee_rules" className="text-sm font-medium">
                          Fee Rules (JSON)
                        </label>
                        <Textarea
                          id="fee_rules"
                          value={feeTableRules}
                          onChange={(e) => setFeeTableRules(e.target.value)}
                          rows={15}
                          className="font-mono text-sm"
                          placeholder='{"platform_fee_percent": "6.5", "transaction_fee": "0.30", "payoneer_fee_percent": "3.0"}'
                        />
                      </div>
                      <p className="text-xs text-muted-foreground">
                        Example structure varies by platform. Refer to platform documentation for required fields.
                      </p>
                    </div>
                    <DialogFooter className="gap-2">
                      <Button variant="outline" onClick={() => { setShowFeeTableDialog(null); setFeeTableRules(''); }}>
                        Cancel
                      </Button>
                      <Button onClick={() => handleUpdateFeeTable(showFeeTableDialog!)} disabled={isUpdatingFeeTable}>
                        {isUpdatingFeeTable ? (
                          <>
                            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                            Saving...
                          </>
                        ) : (
                          <>
                            <Save className="mr-2 h-4 w-4" />
                            Save Rules
                          </>
                        )}
                      </Button>
                    </DialogFooter>
                  </DialogContent>
                </Dialog>
              )}
            </div>
          </TabsContent>

          {/* Webhooks Tab */}
          <TabsContent value="webhooks" className="space-y-6">
            <div className="space-y-4">
              <p className="text-sm text-muted-foreground">
                Webhook status and configuration. Webhooks are verified before any database write (S13).
              </p>
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Globe className="h-5 w-5" />
                    Webhook Endpoints
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  {(['etsy', 'lemon-squeezy', 'printify', 'gelato'] as Platform[]).map((platform) => (
                    <div key={platform} className="flex items-center justify-between p-3 border rounded-lg">
                      <div className="flex items-center gap-3">
                        <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-muted">
                          {platformIcons[platform]}
                        </div>
                        <div>
                          <p className="font-medium">{platformLabels[platform]}</p>
                          <p className="text-sm text-muted-foreground">
                            Signature header verified before processing
                          </p>
                        </div>
                      </div>
                      <Badge variant="outline">Configured in platform dashboard</Badge>
                    </div>
                  ))}
                </CardContent>
              </Card>
            </div>
          </TabsContent>
        </Tabs>
      </CardContent>
    </Card>
  );
}