import { Table as MantineTable } from '@mantine/core';
import { FC, PropsWithChildren } from 'react';

export interface TableProps {
  data: Array<Record<string, string>>;
  columns: Array<{ id: string; label: string }>;
}

export const Table: FC<PropsWithChildren<TableProps>> = ({ data, columns }) => {
  const rows = data.map((item) => (
    <MantineTable.Tr key={item.name}>
      <MantineTable.Td>{item.position}</MantineTable.Td>
      <MantineTable.Td>{item.name}</MantineTable.Td>
      <MantineTable.Td>{item.symbol}</MantineTable.Td>
      <MantineTable.Td>{item.mass}</MantineTable.Td>
    </MantineTable.Tr>
  ));

  return (
    <MantineTable>
      <MantineTable.Thead>
        <MantineTable.Tr>
          {columns.map((column) => (
            <MantineTable.Th key={column.id}>{column.label}</MantineTable.Th>
          ))}
        </MantineTable.Tr>
      </MantineTable.Thead>
      <MantineTable.Tbody>{rows}</MantineTable.Tbody>
    </MantineTable>
  );
};
