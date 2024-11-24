import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { Router } from './App.tsx';
import { AppShell, ThemeProvider } from '@panorama/atoms';
import { dispatchNavEvent } from './pluginNav.ts';

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ThemeProvider>
      <AppShell plugins={[{ id: 'discover', label: 'Discover', to: '/' }]}>
        <Router config={{ basename: '/', pluginId: 'discover' }} />
      </AppShell>
    </ThemeProvider>
  </StrictMode>,
);

setTimeout(
  () => dispatchNavEvent({ pluginId: 'discover', basename: '/' }),
  100,
);
