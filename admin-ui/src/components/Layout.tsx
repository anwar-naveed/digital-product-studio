// Main Layout with navigation sidebar
import { Outlet, NavLink, useNavigate } from 'react-router-dom';
import { useAuth } from '../hooks/useAuth';
import {
  LayoutDashboard,
  ListMusic,
  PlayCircle,
  ClipboardList,
  Package,
  Store,
  BarChart3,
  Settings,
  Grid,
  LogOut,
  ChevronLeft,
  ChevronRight,
} from 'lucide-react';
import { useState } from 'react';
import { cn } from '../lib/utils';

interface NavItem {
  label: string;
  href: string;
  icon: React.ReactNode;
}

const navItems: NavItem[] = [
  { label: 'Dashboard', href: '/dashboard', icon: <LayoutDashboard className="h-5 w-5" /> },
  { label: 'Niches', href: '/niches', icon: <ListMusic className="h-5 w-5" /> },
  { label: 'Runs', href: '/runs', icon: <PlayCircle className="h-5 w-5" /> },
  { label: 'Curation', href: '/curation', icon: <ClipboardList className="h-5 w-5" /> },
  { label: 'Catalog', href: '/catalog', icon: <Package className="h-5 w-5" /> },
  { label: 'Listings', href: '/listings', icon: <Store className="h-5 w-5" /> },
  { label: 'Analytics', href: '/analytics', icon: <BarChart3 className="h-5 w-5" /> },
  { label: 'Niche Board', href: '/niche-board', icon: <Grid className="h-5 w-5" /> },
  { label: 'Settings', href: '/settings', icon: <Settings className="h-5 w-5" /> },
];

export function Layout() {
  const { logout } = useAuth();
  const navigate = useNavigate();
  const [collapsed, setCollapsed] = useState(false);

  const handleLogout = () => {
    logout();
    navigate('/login');
  };

  return (
    <div className="flex h-screen bg-gray-50 dark:bg-gray-900">
      {/* Sidebar */}
      <aside
        className={cn(
          'fixed left-0 top-0 z-40 h-screen border-r bg-white dark:bg-gray-800 transition-all duration-200',
          collapsed ? 'w-16' : 'w-64'
        )}
      >
        <div className="flex h-16 items-center justify-between border-b px-4">
          {!collapsed && (
            <span className="text-xl font-bold text-gray-900 dark:text-white">
              Digital Product Studio
            </span>
          )}
          <button
            onClick={() => setCollapsed(!collapsed)}
            className="p-2 rounded-md text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-700"
            aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
          >
            {collapsed ? (
              <ChevronRight className="h-5 w-5" />
            ) : (
              <ChevronLeft className="h-5 w-5" />
            )}
          </button>
        </div>

        <nav className="flex-1 overflow-y-auto p-4 space-y-1">
          {navItems.map((item) => (
            <NavLink
              key={item.href}
              to={item.href}
              className={({ isActive }) =>
                cn(
                  'flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-colors',
                  isActive
                    ? 'bg-primary text-primary-foreground'
                    : 'text-gray-600 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700',
                  collapsed && 'justify-center'
                )
              }
              title={collapsed ? item.label : undefined}
            >
              {item.icon}
              {!collapsed && <span>{item.label}</span>}
            </NavLink>
          ))}
        </nav>

        <div className="border-t p-4">
          {!collapsed ? (
            <button
              onClick={handleLogout}
              className="flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium text-gray-600 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700"
            >
              <LogOut className="h-5 w-5" />
              <span>Logout</span>
            </button>
          ) : (
            <button
              onClick={handleLogout}
              className="mx-auto rounded-lg p-2.5 text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-700"
              title="Logout"
            >
              <LogOut className="h-5 w-5" />
            </button>
          )}
        </div>
      </aside>

      {/* Main content */}
      <main
        className={cn(
          'flex-1 overflow-auto transition-all duration-200',
          collapsed ? 'ml-16' : 'ml-64'
        )}
      >
        <header className="sticky top-0 z-30 border-b bg-white/95 dark:bg-gray-800/95 backdrop-blur supports-[backdrop-filter]:bg-white/60">
          <div className="flex h-16 items-center justify-between px-6">
            <h1 className="text-xl font-semibold text-gray-900 dark:text-white">
              Admin Console
            </h1>
            <div className="flex items-center gap-4">
              <span className="text-sm text-gray-500 dark:text-gray-400">
                Admin User
              </span>
            </div>
          </div>
        </header>

        <div className="p-6">
          <Outlet />
        </div>
      </main>
    </div>
  );
}