import { MantineProvider } from '@mantine/core';
import { FC, PropsWithChildren } from 'react';

export const ThemeProvider: FC<PropsWithChildren> = ({ children }) => {
  return <MantineProvider>{children}</MantineProvider>;
};
