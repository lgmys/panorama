import {Button as MantineButton} from '@mantine/core';
import { FC, PropsWithChildren } from 'react';

export const Button: FC<PropsWithChildren> = ({children}) => {
  return <MantineButton variant='filled'>{children}</MantineButton>
}
