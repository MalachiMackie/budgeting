import { Button, NumberInput, Select, Table } from "@mantine/core";
import { DatePickerInput } from "@mantine/dates";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import {
  CreateTransactionRequest,
  Payee,
  Transaction,
} from "../api/budgetingApi";
import { useBudgetingApi } from "../App";

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
    mutationKey: ['create-transaction'],
    mutationFn: budgetingApi.createTransaction,
    onSuccess: async () => {
        await queryClient.invalidateQueries({queryKey: ['transactions']})
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
          <Table.Tr>
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

function NewTransactionRow({
  payees,
  save,
}: {
  payees: Payee[];
  save: (x: CreateTransactionRequest) => Promise<void>;
}): JSX.Element {
  // todo: show loading
  const [date, setDate] = useState(new Date());
  const [amount, setAmount] = useState<number | undefined>(undefined);
  const [payeeId, setPayeeId] = useState<string | undefined>(undefined);
  const handleSaveClick = async () => {
    if (!payeeId) {
      alert("Cannot save transaction without a payee");
      return;
    }
    if (typeof amount === "undefined") {
      alert("Cannot save transaction without a transaction amount");
      return;
    }
    // todo: do this better maybe?
    const amountDollars = Math.trunc(amount);
    const amountCents = Math.trunc(Math.abs(amount - amountDollars) * 100);
    try {
      await save({
        date: formatDate(date),
        payee_id: payeeId,
        amount_dollars: amountDollars,
        amount_cents: amountCents,
      });
      setDate(new Date());
      setAmount(undefined);
      setPayeeId(undefined);
    } catch {
      // handle error
    }
  };

  return (
    <>
      <Table.Tr style={{ borderBottom: 0 }}>
        <Table.Td>
          <DatePickerInput value={date} onChange={(x) => x && setDate(x)} />
        </Table.Td>
        <Table.Td>
          <NumberInput
            value={amount}
            onChange={(value) => typeof value === "number" && setAmount(value)}
          />
        </Table.Td>
        <Table.Td>
          <div style={{ display: "flex", flexDirection: "column" }}>
            <Select
              onChange={(x) => x && setPayeeId(x)}
              data={payees.map((x) => ({ value: x.id, label: x.name }))}
              value={payeeId}
            />
          </div>
        </Table.Td>
      </Table.Tr>
      <Table.Tr>
        <Table.Td colSpan={3}>
          <Button onClick={handleSaveClick}>Save</Button>
        </Table.Td>
      </Table.Tr>
    </>
  );
}

function formatDate(date: Date): string {
  let sb: (string | number)[] = [date.getFullYear(), "-"];
  const month = date.getMonth() + 1;
  if (month < 10) {
    sb.push("0");
  }
  sb.push(month, "-");

  const dayOfMonth = date.getDate();
  if (dayOfMonth < 10) {
    sb.push("0");
  }
  sb.push(dayOfMonth);

  return sb.join("");
}
