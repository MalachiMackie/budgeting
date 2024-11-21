import { Table } from "@mantine/core";
import { Payee } from "../api/client";

export type PayeesListProps = { payees: Payee[] };

export function PayeesList({ payees }: PayeesListProps): JSX.Element {
  return (
    <Table>
      <Table.Thead>
        <Table.Tr>
          <Table.Th>Name</Table.Th>
        </Table.Tr>
      </Table.Thead>
      <Table.Tbody>
        {payees.map((x) => (
          <Table.Tr key={x.id}>
            <Table.Td>{x.name}</Table.Td>
          </Table.Tr>
        ))}
      </Table.Tbody>
    </Table>
  );
}
