import './App.css'
import { useCallback } from 'react'

const loadPlugin = (pluginId: string) => {
    const stylesheet = document.createElement('link');
    stylesheet.href = `/plugins/${pluginId}/style.css`;
    stylesheet.rel = 'stylesheet';

    const scriptElement = document.createElement('script');
    scriptElement.type = 'module';
    scriptElement.innerText = `
      const plugin = await import('/plugins/${pluginId}/${pluginId}.js');
      plugin.start(document.querySelector('#plugin-host'));
    `;

    document.head.appendChild(stylesheet);
    document.head.appendChild(scriptElement);
}

function App() {
  const onLoad = useCallback(() => {
    loadPlugin('discover');
  }, []);

  return (
    <>
      <button onClick={onLoad}>Load discover plugin</button>

      <div id="plugin-host"></div>
    </>
  )
}

export default App
