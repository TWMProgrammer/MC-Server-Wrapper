import { useState, useEffect } from 'react';

export type AccentColor = {
  name: string;
  value: string; // HSL value without 'hsl()' prefix, e.g. "221.2 83.2% 53.3%"
};

export const ACCENT_COLORS: AccentColor[] = [
  { name: 'Blue', value: '221.2 83.2% 53.3%' },
  { name: 'Emerald', value: '160 84% 39%' },
  { name: 'Rose', value: '346 84% 61%' },
  { name: 'Amber', value: '38 92% 50%' },
  { name: 'Indigo', value: '239 84% 67%' },
  { name: 'Violet', value: '262 83% 58%' },
];

export type Theme = 'dark' | 'light';

export function useAppSettings() {
  const [accentColor, setAccentColor] = useState<AccentColor>(() => {
    const saved = localStorage.getItem('app-accent-color');
    if (saved) {
      try {
        const parsed = JSON.parse(saved);
        return ACCENT_COLORS.find(c => c.name === parsed.name) || ACCENT_COLORS[0];
      } catch (e) {
        return ACCENT_COLORS[0];
      }
    }
    return ACCENT_COLORS[0];
  });

  const [theme, setTheme] = useState<Theme>(() => {
    const saved = localStorage.getItem('app-theme');
    if (saved === 'dark' || saved === 'light') return saved;
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  });

  const [scaling, setScaling] = useState<number>(() => {
    const saved = localStorage.getItem('app-scaling');
    if (saved) {
      const parsed = parseFloat(saved);
      return isNaN(parsed) ? 1.0 : parsed;
    }
    return 1.0;
  });

  useEffect(() => {
    const root = document.documentElement;
    root.style.setProperty('--primary', accentColor.value);
    
    // Calculate hover and active colors (roughly)
    const [h, s, l] = accentColor.value.split(' ');
    const lValue = parseFloat(l.replace('%', ''));
    
    root.style.setProperty('--primary-hover', `${h} ${s} ${lValue - 8}%`);
    root.style.setProperty('--primary-active', `${h} ${s} ${lValue - 18}%`);
    
    localStorage.setItem('app-accent-color', JSON.stringify(accentColor));
  }, [accentColor]);

  useEffect(() => {
    const root = document.documentElement;
    if (theme === 'dark') {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }
    localStorage.setItem('app-theme', theme);
  }, [theme]);

  useEffect(() => {
    localStorage.setItem('app-scaling', scaling.toString());
  }, [scaling]);

  return {
    accentColor,
    setAccentColor,
    theme,
    setTheme,
    scaling,
    setScaling,
  };
}
