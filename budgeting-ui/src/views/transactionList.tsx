import { Table } from "@mantine/core";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { CreateTransactionRequest, Transaction } from "../api/budgetingApi";
import { useBudgetingApi } from "../App";
import { queryKeys } from "../queryKeys";
import { NewTransactionRow } from "./NewTransactionRow";
import { TransactionRow } from "./TransactionRow";

export function TransactionList({
  bankAccountId,
  userId,
}: {
  bankAccountId: string;
  userId: string;
}): JSX.Element {
  const budgetingApi = useBudgetingApi();
  const queryClient = useQueryClient();

  const [selectedRows, setSelectedRows] = useState<Set<string>>(new Set());
  const [editingRow, setEditingRow] = useState<string | null>(null);

  const { data: payees, isLoading: payeesLoading } = useQuery({
    queryKey: queryKeys.payees.fetch(userId),
    queryFn: () => budgetingApi.getPayees(userId),
  });

  const { data: budgets, isLoading: budgetsLoading } = useQuery({
    queryKey: queryKeys.budgets.fetch,
    queryFn: () => budgetingApi.getBudgets(userId),
  });

  const { data: transactions, isLoading: transactionsLoading } = useQuery({
    queryKey: queryKeys.transactions.fetch(bankAccountId),
    queryFn: () => budgetingApi.getTransactions(bankAccountId),
  });

  const createTransaction = useMutation({
    mutationKey: queryKeys.transactions.create,
    mutationFn: (request: CreateTransactionRequest) =>
      budgetingApi.createTransaction(request, bankAccountId),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.transactions.fetch(bankAccountId),
      });
    },
  });

  const loading = transactionsLoading || payeesLoading || budgetsLoading;

  if (loading) {
    return <>Loading...</>;
  }

  if (!payees || !transactions || !budgets) {
    return <>Failed to load things</>;
  }

  let payeesMap = new Map(payees.map((x) => [x.id, x]));
  let budgetsMap = new Map(budgets.map((x) => [x.id, x]));

  const compareTransactions = (a: Transaction, b: Transaction): number => {
    // todo: within the same day, compare by amount

    // sort descending date
    return b.date.localeCompare(a.date);
  };

  return (
    <>
      <Table>
        <Table.Thead>
          <Table.Tr>
            <Table.Th></Table.Th>
            <Table.Th>Date</Table.Th>
            <Table.Th>Budget</Table.Th>
            <Table.Th>Amount</Table.Th>
            <Table.Th>Payee</Table.Th>
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          <NewTransactionRow
            save={createTransaction.mutateAsync}
            payees={payees}
            budgets={budgets}
          />
          {[...transactions].sort(compareTransactions).map((x) => (
            <TransactionRow
              selected={selectedRows.has(x.id)}
              onRowSelected={() => {
                setEditingRow(null);
                setSelectedRows(new Set([x.id]));
              }}
              onCheckboxSelectedChange={(selected) => {
                setEditingRow(null);
                setSelectedRows((prev) => {
                  !selected && prev.delete(x.id);
                  selected && prev.add(x.id);
                  return new Set(prev);
                });
              }}
              isEdit={editingRow === x.id}
              onEditChange={(isEditing) => {
                if (isEditing && selectedRows.has(x.id)) {
                  setEditingRow(x.id);
                  // if editing, ensure only this row is selected
                  setSelectedRows(new Set([x.id]));
                }
                if (!isEditing && editingRow === x.id) {
                  setEditingRow(null);
                }
              }}
              key={x.id}
              transaction={x}
              payeesMap={payeesMap}
              budgetsMap={budgetsMap}
              bankAccountId={bankAccountId}
            />
          ))}
          {transactions.length === 0 && (
            <Table.Tr>
              <Table.Td colSpan={3}>No transactions</Table.Td>
            </Table.Tr>
          )}
        </Table.Tbody>
      </Table>
    </>
  );
}
