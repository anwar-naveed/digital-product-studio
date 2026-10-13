// Test setup for vitest
import '@testing-library/jest-dom';
import { vi } from 'vitest';

// Mock react-router-dom (importActual — vitest 3 API)
const routerActual = await vi.importActual('react-router-dom');
vi.mock('react-router-dom', () => ({
  ...routerActual,
  useNavigate: () => vi.fn(),
  useLocation: () => ({ pathname: '/', search: '', hash: '', state: null }),
  useParams: () => ({}),
  NavLink: ({ children, to, className, ...props }: any) => (
    <a href={to} className={typeof className === 'function' ? className({ isActive: false }) : className} {...props}>
      {children}
    </a>
  ),
}));

// NOTE: @tanstack/react-query is NOT globally mocked — hooks tests rely on the REAL
// implementation with a fresh QueryClient (see createWrapper in each test).
// Component tests that need stubs mock it locally in their own files.

// Mock lucide-react icons — every icon imported anywhere in src/ gets a stub <svg>.
// Regenerate this list with: grep -rh "import .* lucide-react" src --include="*.tsx" | grep -oE "\{[^}]*\}" | tr -d '{}' | tr ',' '\n' | sed 's/^ *//;s/ *$//' | sort -u
vi.mock('lucide-react', () => {
  const icons = [
    'AlertCircle',
    'AlertTriangle',
    'ArrowRight',
    'ArrowUpDown',
    'BarChart3',
    'Box',
    'Check',
    'CheckCircle',
    'ChevronDown',
    'ChevronLeft',
    'ChevronRight',
    'ChevronUp',
    'ClipboardList',
    'Clock',
    'DollarSign',
    'Download',
    'Edit',
    'Eye',
    'EyeOff',
    'FileText',
    'Filter',
    'Globe',
    'Grid',
    'GripVertical',
    'Heart',
    'Image',
    'Key',
    'Layers',
    'LayoutDashboard',
    'ListMusic',
    'Loader2',
    'LogOut',
    'Package',
    'Play',
    'PlayCircle',
    'Plus',
    'Printer',
    'RefreshCw',
    'Save',
    'Search',
    'Send',
    'Settings',
    'Shield',
    'ShoppingCart',
    'SlidersHorizontal',
    'Square',
    'Store',
    'Tag',
    'Target',
    'Trash2',
    'TrendingDown',
    'TrendingUp',
    'X',
    'XCircle',
    'Zap',
  ];

  const mockIcon = (name: string) => {
    const Component = ({ className, ...props }: any) => (
      <svg className={className} {...props} data-testid={`icon-${name.toLowerCase()}`} />
    );
    (Component as any).displayName = name;
    return Component;
  };

  const exports: Record<string, any> = {};
  for (const name of icons) exports[name] = mockIcon(name);
  exports.default = undefined;
  return exports;
});

// Mock clsx and tailwind-merge
vi.mock('clsx', () => ({
  clsx: (...args: any[]) => args.filter(Boolean).join(' '),
}));

vi.mock('tailwind-merge', () => ({
  twMerge: (...args: string[]) => args.join(' '),
}));

// Mock react-hook-form
vi.mock('react-hook-form', () => ({
  useForm: () => ({
    register: vi.fn(),
    handleSubmit: vi.fn((fn) => (e: any) => { e.preventDefault(); fn({}); }),
    reset: vi.fn(),
    formState: { errors: {}, isSubmitting: false },
    watch: vi.fn(),
  }),
}));

vi.mock('@hookform/resolvers/zod', () => ({
  zodResolver: vi.fn(() => (values: any) => Promise.resolve({ values, errors: {} })),
}));

// Mock api client
// NOTE: api client is NOT globally mocked — each test file mocks it locally.


// Global test utilities
(globalThis as any).ResizeObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}));

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});
// jsdom pointer + scroll stubs required by radix primitives
if (!Element.prototype.scrollIntoView) {
  Element.prototype.scrollIntoView = () => {};
}
if (!(Element.prototype as any).hasPointerCapture) {
  (Element.prototype as any).hasPointerCapture = () => false;
  (Element.prototype as any).setPointerCapture = () => {};
  (Element.prototype as any).releasePointerCapture = () => {};
}
