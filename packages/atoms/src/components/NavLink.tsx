import { NavLink as MantineNavLink } from '@mantine/core';
import { FC } from 'react';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const NavLink: FC<{ to: string; label: string; component: any }> = ({
  label,
  to,
  component,
}) => {
  return <MantineNavLink to={to} component={component} label={label} />;
};
