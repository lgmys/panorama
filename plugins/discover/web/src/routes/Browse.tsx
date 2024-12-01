import { Table } from '@panorama/atoms';
import { FC } from 'react';

export const Browse: FC = () => {
  return (
    <div>
      <Table data={[]} columns={[{ id: 'name', label: 'Name' }]} />
    </div>
  );
};
