import { Table } from "@mantine/core";
import { useNavigate } from "react-router-dom";
import { BankAccount } from "../api/client";

export function BankAccountList({
  bankAccounts,
}: {
  bankAccounts: BankAccount[];
}): JSX.Element {
  const navigate = useNavigate();

  return (
    <>
      <Table>
        <Table.Thead>
          <Table.Tr>
            <Table.Th>Name</Table.Th>
            <Table.Th>Balance</Table.Th>
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {bankAccounts?.map((x) => (
            <Table.Tr key={x.id} onClick={() => navigate(`/accounts/${x.id}`)}>
              <Table.Td>{x.name}</Table.Td>
              <Table.Td>${x.balance.toFixed(2)}</Table.Td>
            </Table.Tr>
          ))}
        </Table.Tbody>
      </Table>
    </>
  );
}
