export const loadPlugin = (pluginId: string) => {
    const pluginHost = document.querySelector('#plugin-host')!;
    pluginHost.attachShadow({mode: 'open'});

    const pluginWrapper = document.createElement('div');
    pluginWrapper.setAttribute('id', 'plugin-wrapper');

    pluginHost.shadowRoot?.appendChild(pluginWrapper);

    // @ts-expect-error ignore
    window.pluginWrapper = pluginWrapper;

    const stylesheet = document.createElement('link');
    stylesheet.href = `/plugins/${pluginId}/style.css`;
    stylesheet.rel = 'stylesheet';

    const scriptElement = document.createElement('script');
    scriptElement.type = 'module';
    scriptElement.innerText = `
      const plugin = await import('/plugins/${pluginId}/${pluginId}.js');
      plugin.start(window.pluginWrapper, {pluginId: '${pluginId}', basename: '/app/${pluginId}'});
    `;

    document.head.appendChild(stylesheet);
    document.head.appendChild(scriptElement);
}
