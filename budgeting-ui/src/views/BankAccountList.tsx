import { Table, Title } from "@mantine/core";
import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { useBudgetingApi } from "../App";
import TransactionList from "./transactionList";

export function BankAccountList({ userId }: { userId: string }): JSX.Element {
  const api = useBudgetingApi();
  const { data: bankAccounts } = useQuery({
    queryKey: ["bank-accounts", userId],
    queryFn: () => api.getBankAccounts(userId),
  });

  const [selectedBankAccountId, setSelectedBankAccountId] = useState<
    string | null
  >(null);

  return (
    <>
      <Title>Bank Accounts</Title>
      <Table>
        <Table.Thead>
          <Table.Tr>
            <Table.Th>Name</Table.Th>
            <Table.Th>Starting Balance</Table.Th>
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {bankAccounts?.map((x) => (
            <Table.Tr onClick={() => setSelectedBankAccountId(x.id)}>
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
