import { Table } from "@mantine/core";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Transaction } from "../api/budgetingApi";
import { useBudgetingApi } from "../App";
import { NewTransactionRow } from "./NewTransactionRow";

export default function TransactionList(): JSX.Element {
  const budgetingApi = useBudgetingApi();
  const queryClient = useQueryClient();

  const { data: payees, isLoading: payeesLoading } = useQuery({
    queryKey: ["payees"],
    queryFn: budgetingApi.getPayees,
  });
  const { data: transactions, isLoading: transactionsLoading } = useQuery({
    queryKey: ["transactions"],
    queryFn: budgetingApi.getTransactions,
  });
  const createTransaction = useMutation({
    mutationKey: ["create-transaction"],
    mutationFn: budgetingApi.createTransaction,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["transactions"] });
    },
  });

  const loading = transactionsLoading || payeesLoading;

  if (loading) {
    return <>Loading...</>;
  }

  if (!payees || !transactions) {
    return <>Failed to load things</>;
  }

  let payeesMap = new Map(payees.map((x) => [x.id, x]));

  const headers = [
    { key: "date", header: "Date" },
    { key: "amount", header: "Amount" },
    { key: "payee", header: "Payee" },
  ];

  const compareTransactions = (a: Transaction, b: Transaction): number => {
    // todo: within the same day, compare by amount

    // sort descending date
    return b.date.localeCompare(a.date);
  };

  return (
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
        />
        {[...transactions].sort(compareTransactions).map((x) => (
          <Table.Tr key={x.id}>
            <Table.Td>{new Date(x.date).toDateString()}</Table.Td>
            <Table.Td>
              $
              {(
                x.amount_dollars +
                (x.amount_cents * Math.sign(x.amount_dollars)) / 100
              ).toFixed(2)}
            </Table.Td>
            <Table.Td>{payeesMap.get(x.payee_id)?.name}</Table.Td>
          </Table.Tr>
        ))}
      </Table.Tbody>
    </Table>
  );
}
