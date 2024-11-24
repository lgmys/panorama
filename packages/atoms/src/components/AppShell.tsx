import {
  FC,
  PropsWithChildren,
  ReactElement,
  useEffect,
  useMemo,
  useState,
} from 'react';
import {
  AccordionChevron,
  Container,
  Loader,
  AppShell as MantineAppShell,
  NavLink,
  Text,
} from '@mantine/core';

import { PLUGIN_EVENTS, PluginNavigationInit } from '@panorama/shared-types';

export const AppShell: FC<
  PropsWithChildren<{
    insideHostApp?: boolean;
    navPre?: ReactElement;
    navPost?: ReactElement;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    navLinkComponent?: any;
    plugins: Array<{ label: string; to: string; id: string }>;
  }>
> = ({ children, plugins, navLinkComponent, navPre, navPost }) => {
  const [pluginNavigation, setPluginNavigation] = useState<
    PluginNavigationInit | undefined
  >();

  const [loadingPlugin, setLoadingPlugin] = useState(false);

  useEffect(() => {
    const handlePluginLoading = (event: unknown) => {
      if (!(event instanceof CustomEvent)) {
        return;
      }

      setLoadingPlugin(true);
    };

    window.addEventListener(PLUGIN_EVENTS.LOADING, handlePluginLoading);

    return () =>
      window.removeEventListener(PLUGIN_EVENTS.LOADING, handlePluginLoading);
  }, []);

  useEffect(() => {
    const handlePluginUnload = (event: unknown) => {
      if (!(event instanceof CustomEvent)) {
        return;
      }

      setPluginNavigation(undefined);
    };

    window.addEventListener(PLUGIN_EVENTS.UNLOAD, handlePluginUnload);

    return () =>
      window.removeEventListener(PLUGIN_EVENTS.UNLOAD, handlePluginUnload);
  }, []);

  useEffect(() => {
    const handlePluginLoad = (event: unknown) => {
      if (!(event instanceof CustomEvent)) {
        return;
      }

      const { detail } = event as CustomEvent<PluginNavigationInit>;

      setLoadingPlugin(false);
      setPluginNavigation(detail);
    };

    window.addEventListener(PLUGIN_EVENTS.INIT_NAVIGATION, handlePluginLoad);

    return () =>
      window.removeEventListener(
        PLUGIN_EVENTS.INIT_NAVIGATION,
        handlePluginLoad,
      );
  }, []);

  const navigationItems = useMemo(() => {
    return plugins.map((plugin) => {
      if (pluginNavigation?.pluginId === plugin.id) {
        return (
          <NavLink key={plugin.id} label={plugin.label} defaultOpened>
            {pluginNavigation?.items.map((item) => {
              return (
                <NavLink
                  onClick={(e) => {
                    e.preventDefault();
                    window.dispatchEvent(
                      new CustomEvent(PLUGIN_EVENTS.NAVIGATE, {
                        detail: { ...item, plugin },
                      }),
                    );
                  }}
                  key={item.to}
                  label={item.label}
                />
              );
            })}
          </NavLink>
        );
      }

      return (
        <NavLink
          key={plugin.to}
          to={plugin.to}
          component={navLinkComponent ?? 'a'}
          label={plugin.label}
          rightSection={
            loadingPlugin ? (
              <Loader color="blue" size="xs" />
            ) : (
              <AccordionChevron style={{ transform: 'rotate(-90deg)' }} />
            )
          }
        />
      );
    });
  }, [
    loadingPlugin,
    navLinkComponent,
    pluginNavigation?.items,
    pluginNavigation?.pluginId,
    plugins,
  ]);

  return (
    <MantineAppShell
      navbar={{
        width: '8rem',
        breakpoint: 'sm',
        collapsed: {
          desktop: false,
          mobile: false,
        },
      }}
      header={{
        height: '2.5rem',
      }}
    >
      <MantineAppShell.Header>
        <Text
          size="xs"
          pt={12}
          pl={8}
          fw={700}
          style={{ textTransform: 'uppercase' }}
        >
          panorama
        </Text>
      </MantineAppShell.Header>
      <MantineAppShell.Navbar>
        {navPre}
        {navigationItems}
        {navPost}
      </MantineAppShell.Navbar>
      <MantineAppShell.Main>
        <Container fluid pt={4}>
          {children}
        </Container>
      </MantineAppShell.Main>
    </MantineAppShell>
  );
};
