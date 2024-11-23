import './App.css';
import { FC, useEffect, useRef } from 'react';
import {
  BrowserRouter,
  Outlet,
  Route,
  Routes,
  useParams,
  Link,
} from 'react-router';
import { loadPlugin } from './loadPlugin';

import { AppShell, Button, NavLink, ThemeProvider } from '@panorama/atoms';

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

const MenuItem: FC<{ to: string; label: string }> = ({ to, label }) => {
  return <NavLink label={label} component={Link} to={to} />;
};

const MenuPrimary = () => {
  return (
    <nav>
      <MenuItem label="Home" to={routes.HOME} />

      {plugins.map((plugin) => (
        <MenuItem to={pluginRoute(plugin)} label={plugin.label} />
      ))}
    </nav>
  );
};

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
    <AppShell nav={<MenuPrimary />}>
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
