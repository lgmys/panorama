import { FC, PropsWithChildren, useEffect, useMemo, useState } from 'react';
import {
  Container,
  AppShell as MantineAppShell,
  NavLink,
  Text,
} from '@mantine/core';
import { PLUGIN_EVENTS, PluginNavigationInit } from '@panorama/shared-types';

export const AppShell: FC<
  PropsWithChildren<{
    plugins: Array<{ label: string; to: string; id: string }>;
  }>
> = ({ children, plugins }) => {
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
          <NavLink label={plugin.label}>
            {pluginNavigation?.items.map((item) => {
              return (
                <NavLink
                  onClick={() =>
                    window.dispatchEvent(
                      new CustomEvent(PLUGIN_EVENTS.NAVIGATE, { detail: item }),
                    )
                  }
                  key={item.to}
                  label={item.label}
                />
              );
            })}
          </NavLink>
        );
      }

      return <NavLink href={plugin.to} component="a" label={plugin.label} />;
    });
  }, [pluginNavigation, plugins]);

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
      <MantineAppShell.Navbar>{navigationItems}</MantineAppShell.Navbar>
      <MantineAppShell.Main>
        <Container fluid pt={4}>
          {children}
        </Container>
      </MantineAppShell.Main>
    </MantineAppShell>
  );
};
