import { Table } from "@mantine/core";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { CreateTransactionRequest, Transaction } from "../api/budgetingApi";
import { useBudgetingApi } from "../App";
import { queryKeys } from "../queryKeys";
import { NewTransactionRow } from "./NewTransactionRow";

export default function TransactionList({
  bankAccountId,
  userId,
}: {
  bankAccountId: string;
  userId: string;
}): JSX.Element {
  const budgetingApi = useBudgetingApi();
  const queryClient = useQueryClient();

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

  const headers = [
    { key: "date", header: "Date" },
    { key: "budget", header: "Budget" },
    { key: "amount", header: "Amount" },
    { key: "payee", header: "Payee" },
  ];

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
            {headers.map((x) => (
              <Table.Th key={x.key}>{x.header}</Table.Th>
            ))}
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          <NewTransactionRow
            save={createTransaction.mutateAsync}
            payees={payees}
            budgets={budgets}
          />
          {[...transactions].sort(compareTransactions).map((x) => (
            <Table.Tr key={x.id}>
              <Table.Td>{new Date(x.date).toDateString()}</Table.Td>
              <Table.Td>{budgetsMap.get(x.budget_id)?.name}</Table.Td>
              <Table.Td>${x.amount.toFixed(2)}</Table.Td>
              <Table.Td>{payeesMap.get(x.payee_id)?.name}</Table.Td>
            </Table.Tr>
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
