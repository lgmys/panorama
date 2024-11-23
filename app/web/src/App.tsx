import './App.css';
import { useEffect, useRef } from 'react';
import { BrowserRouter, Outlet, Route, Routes, useParams } from 'react-router';
import { loadPlugin } from './loadPlugin';

import { AppShell, Button, ThemeProvider } from '@panorama/atoms';

interface Plugin {
  id: string;
  label: string;
}

const plugins: Plugin[] = [
  {
    id: 'discover',
    label: 'Discover',
  },
];

const BASE = '/app';

const routes = {
  HOME: `${BASE}/`,
};

const pluginRoute = (plugin: Plugin) => `${BASE}/${plugin.id}`;

const PluginRoot = () => {
  const params = useParams();
  const loadedRef = useRef<boolean>();

  useEffect(() => {
    if (!params.pluginId) {
      return;
    }

    if (!loadedRef.current) {
      loadPlugin(params.pluginId);
      loadedRef.current = true;
    }
  }, [params.pluginId]);

  return <div id="plugin-host"></div>;
};

const Home = () => {
  return (
    <div>
      <Button>Home</Button>
    </div>
  );
};

const App = () => {
  return (
    <AppShell
      plugins={plugins.map((plugin) => ({
        label: plugin.label,
        to: pluginRoute(plugin),
        id: plugin.id,
      }))}
    >
      <Outlet />
    </AppShell>
  );
};

export const Router = () => {
  return (
    <ThemeProvider>
      <BrowserRouter>
        <Routes>
          <Route path={BASE} Component={App}>
            <Route Component={Home} path={routes.HOME} />
            <Route path=":pluginId/*" Component={PluginRoot} />
          </Route>
        </Routes>
      </BrowserRouter>
    </ThemeProvider>
  );
};
