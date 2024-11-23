import { FC, PropsWithChildren, ReactNode } from "react";
import {Container, AppShell as MantineAppShell, Text} from '@mantine/core';

export const AppShell: FC<PropsWithChildren<{nav: ReactNode}>> = ({children, nav}) => {
  return (
    <MantineAppShell navbar={
    {
      width: '8rem', 
      breakpoint: 'sm',
      collapsed: {
        desktop: false,
        mobile: false
      }
    }
  }

      header={
        {
          height: '2.5rem'
        }
      }
    >
    <MantineAppShell.Header>
      <Text fw={700}>Panorama</Text>
    </MantineAppShell.Header>
    <MantineAppShell.Navbar>{nav}</MantineAppShell.Navbar>
    <MantineAppShell.Main>
        <Container fluid pt={4}>
          {children}
        </Container>
    </MantineAppShell.Main>
  </MantineAppShell>);
}
