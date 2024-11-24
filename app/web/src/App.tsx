/* eslint-disable @typescript-eslint/no-explicit-any */
import './App.css';
import { useEffect, useRef } from 'react';
import {
  BrowserRouter,
  Link,
  Outlet,
  Route,
  Routes,
  useNavigate,
  useParams,
} from 'react-router';
import { loadPlugin } from './loadPlugin';

import { AppShell, Button, NavLink, ThemeProvider } from '@panorama/atoms';

import { PLUGIN_EVENTS, PluginNavigate } from '@panorama/shared-types';

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
  const navigate = useNavigate();

  useEffect(() => {
    const handler = (event: unknown) => {
      if (!(event instanceof CustomEvent)) {
        return;
      }

      const navigationEvent = event as CustomEvent<
        PluginNavigate & { plugin: { to: string } }
      >;

      console.log(navigationEvent.detail);

      navigate(navigationEvent.detail.plugin.to);
    };

    window.addEventListener(PLUGIN_EVENTS.NAVIGATE, handler);

    return () => window.removeEventListener(PLUGIN_EVENTS.NAVIGATE, handler);
  }, [navigate]);

  return (
    <AppShell
      navPre={<NavLink component={Link as any} label="Home" to={routes.HOME} />}
      navLinkComponent={Link as any}
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
          <Route path={BASE} element={<App />}>
            <Route path=":pluginId/*" element={<PluginRoot />} />
            <Route element={<Home />} index />
          </Route>
        </Routes>
      </BrowserRouter>
    </ThemeProvider>
  );
};
