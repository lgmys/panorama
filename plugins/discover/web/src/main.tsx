import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { Router } from './App.tsx';
import { AppShell, ThemeProvider } from '@panorama/atoms';

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ThemeProvider>
      <AppShell nav={null}>
        <Router />
      </AppShell>
    </ThemeProvider>
  </StrictMode>,
);
