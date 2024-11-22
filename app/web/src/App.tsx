import './App.css'
import { useEffect, useRef } from 'react'
import { BrowserRouter, Link, Outlet, Route, Routes, useParams } from "react-router";
import { loadPlugin } from './loadPlugin';

interface Plugin {
  id: string,
}

const plugins: Plugin[] = [
  {
    id: 'discover',
  }
];

const MenuPrimary = () => {
  return (
    <nav>
      <div><Link to={'/app'}>Home</Link></div>

      {plugins.map(plugin => <div key={plugin.id} ><Link to={`/app/${plugin.id}`}>{plugin.id}</Link></div>)}
    </nav>
  );
}

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

  return <div id="plugin-host"></div>
}

const Home = () => {
  return <div>Home</div>;
}

const App = () => {
  return (
      <div style={{display: 'flex'}}>
        <MenuPrimary />
        <Outlet />
      </div>
  )
}

export const Router = () => {
  return (
    <BrowserRouter>
      <Routes>
        <Route path='/app' Component={App}>
          <Route Component={Home} path='/app/' />
          <Route path=":pluginId/*" Component={PluginRoot} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}



