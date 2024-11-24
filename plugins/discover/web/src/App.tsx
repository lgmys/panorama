import { FC, useEffect } from 'react';
import './App.css';
import {
  BrowserRouter,
  Link,
  Outlet,
  Route,
  Routes,
  useNavigate,
} from 'react-router';
import { PLUGIN_EVENTS, PluginNavigate } from '@panorama/shared-types';

export interface RouterProps {
  basename?: string;
}

export const Router: FC<RouterProps> = (props) => {
  return (
    <BrowserRouter basename={props.basename}>
      <Routes>
        <Route path="/" Component={App}>
          <Route path="/test" Component={() => <div>nested test</div>} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
};

function App() {
  const navigate = useNavigate();

  useEffect(() => {
    const pluginNavigationEventHandler = (event: unknown) => {
      const { detail } = event as CustomEvent<PluginNavigate>;
      console.log('pluginNavigationEventHandler', detail);
      navigate(detail.to);
    };

    window.addEventListener(
      PLUGIN_EVENTS.NAVIGATE,
      pluginNavigationEventHandler,
    );

    return () => {
      window.removeEventListener(
        PLUGIN_EVENTS.NAVIGATE,
        pluginNavigationEventHandler,
      );
    };
  }, [navigate]);

  return (
    <>
      <h1>Discover</h1>
      <div>
        <nav>
          <Link to={'/test'}>go to test</Link>
        </nav>
        <Outlet />
      </div>
    </>
  );
}
