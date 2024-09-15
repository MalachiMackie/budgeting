import { Table } from "@mantine/core";
import { useState } from "react";
import { BankAccount } from "../api/budgetingApi";
import TransactionList from "./transactionList";

export function BankAccountList({
  userId,
  bankAccounts,
}: {
  userId: string;
  bankAccounts: BankAccount[];
}): JSX.Element {
  const [selectedBankAccountId, setSelectedBankAccountId] = useState<
    string | null
  >(null);

  return (
    <>
      <Table>
        <Table.Thead>
          <Table.Tr>
            <Table.Th>Name</Table.Th>
            <Table.Th>Starting Balance</Table.Th>
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {bankAccounts?.map((x) => (
            <Table.Tr key={x.id} onClick={() => setSelectedBankAccountId(x.id)}>
              <Table.Td>{x.name}</Table.Td>
              <Table.Td>{x.initial_amount}</Table.Td>
            </Table.Tr>
          ))}
        </Table.Tbody>
      </Table>
      {!!selectedBankAccountId && (
        <TransactionList
          bankAccountId={selectedBankAccountId}
          userId={userId}
        />
      )}
    </>
  );
}
