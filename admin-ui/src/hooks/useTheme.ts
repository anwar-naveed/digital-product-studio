import { useEffect, useState } from 'react';

export type Theme = 'light' | 'dark';

const KEY = 'dps-theme';

function initial(): Theme {
  const saved = sessionStorage.getItem(KEY) as Theme | null;
  if (saved === 'light' || saved === 'dark') return saved;
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

/** Light/dark theme with `.dark` class on <html> (Tailwind class strategy). */
export function useTheme(): [Theme, () => void] {
  const [theme, setTheme] = useState<Theme>(initial);

  useEffect(() => {
    document.documentElement.classList.toggle('dark', theme === 'dark');
    sessionStorage.setItem(KEY, theme);
  }, [theme]);

  const toggle = () => setTheme((t) => (t === 'dark' ? 'light' : 'dark'));
  return [theme, toggle];
}
