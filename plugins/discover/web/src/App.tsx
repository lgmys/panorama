import { FC } from 'react'
import './App.css'
import {BrowserRouter, Link, Outlet, Route, Routes} from 'react-router';

export interface RouterProps {
  basename?: string;
}

export const Router: FC<RouterProps> = (props) => {
  return (
    <BrowserRouter basename={props.basename}>
      <Routes>
        <Route path='/' Component={App}>
          <Route path='/test' Component={() => <div>nested test</div>} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
};

function App() {
  return (
    <>
      <h1>Discover</h1>
      <div>
        <nav><Link to={'/test'}>go to test</Link></nav>
        <Outlet />
      </div>
    </>
  )
}

