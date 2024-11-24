/* eslint-disable @typescript-eslint/no-explicit-any */
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
  AppShell as MantineAppShell,
  NavLink,
  Text,
} from '@mantine/core';

import { PLUGIN_EVENTS, PluginNavigationInit } from '@panorama/shared-types';

export const AppShell: FC<
  PropsWithChildren<{
    navPre?: ReactElement;
    navPost?: ReactElement;
    navLinkComponent?: ReactElement;
    plugins: Array<{ label: string; to: string; id: string }>;
  }>
> = ({ children, plugins, navLinkComponent, navPre, navPost }) => {
  const [pluginNavigation, setPluginNavigation] = useState<
    PluginNavigationInit | undefined
  >();

  useEffect(() => {
    window.addEventListener(
      PLUGIN_EVENTS.INIT_NAVIGATION,
      (event: unknown) => {
        if (!(event instanceof CustomEvent)) {
          return;
        }

        const { detail } = event as CustomEvent<PluginNavigationInit>;

        setPluginNavigation(detail);
      },
      { once: true },
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
          component={(navLinkComponent as any) ?? 'a'}
          label={plugin.label}
          rightSection={
            <AccordionChevron style={{ transform: 'rotate(-90deg)' }} />
          }
        />
      );
    });
  }, [
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
