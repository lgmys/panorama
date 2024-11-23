export const loadPlugin = (pluginId: string) => {
  const stylesheet = document.createElement('link');
  stylesheet.href = `/plugins/${pluginId}/style.css`;
  stylesheet.rel = 'stylesheet';

  const scriptElement = document.createElement('script');
  scriptElement.type = 'module';
  scriptElement.innerText = `
      const plugin = await import('/plugins/${pluginId}/${pluginId}.js');
      plugin.createRouter(window.pluginWrapper, {pluginId: '${pluginId}', basename: '/app/${pluginId}'});
    `;

  const pluginHost = document.querySelector('#plugin-host')!;
  pluginHost.attachShadow({ mode: 'open' });

  const plugin = document.createElement('div');
  plugin.appendChild(stylesheet);

  const pluginInner = document.createElement('div');

  // @ts-expect-error ignore
  window.pluginWrapper = pluginInner;

  plugin.appendChild(scriptElement);
  plugin.appendChild(pluginInner);

  pluginHost.shadowRoot?.appendChild(plugin);
};
