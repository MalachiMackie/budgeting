import { Button, Checkbox, NumberInput, Select, Table } from "@mantine/core";
import { DatePickerInput } from "@mantine/dates";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { Budget, Payee, Transaction } from "../api/client";
import { useBudgetingApi } from "../App";
import { queryKeys } from "../queryKeys";
import { formatDate } from "../utils/formatDate";

export type TransactionRowProps = {
  selected: boolean;
  onCheckboxSelectedChange: (selected: boolean) => void;
  onRowSelected: () => void;
  onEditChange: (editing: boolean) => void;
  isEdit: boolean;
  transaction: Transaction;
  budgetsMap: Map<string, Budget>;
  payeesMap: Map<string, Payee>;
  bankAccountId: string;
};

export function TransactionRow({
  selected,
  onCheckboxSelectedChange,
  onRowSelected,
  onEditChange,
  isEdit,
  transaction,
  budgetsMap,
  payeesMap,
  bankAccountId,
}: TransactionRowProps): JSX.Element {
  const [dateEdit, setDateEdit] = useState(new Date(transaction.date));
  const [budgetIdEdit, setBudgetIdEdit] = useState(transaction.budget_id);
  const [amountEdit, setAmountEdit] = useState(transaction.amount);
  const [payeeIdEdit, setPayeeIdEdit] = useState(transaction.payee_id);

  const api = useBudgetingApi();
  const queryClient = useQueryClient();

  const saveTransaction = useMutation({
    mutationKey: queryKeys.transactions.edit(transaction.id),
    mutationFn: () => {
      return api.updateTransaction(
        {
          transactionId: transaction.id,
          bankAccountId: transaction.bank_account_id,
        },
        {
          amount: amountEdit,
          budget_id: budgetIdEdit,
          payee_id: payeeIdEdit,
          date: formatDate(dateEdit),
        }
      );
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.transactions.fetch(bankAccountId),
      });
      await Promise.all([
        queryClient.invalidateQueries({
          queryKey: queryKeys.transactions.fetch(bankAccountId),
        }),
        queryClient.invalidateQueries({
          queryKey: queryKeys.bankAccounts.fetchSingle(bankAccountId),
        }),
      ]);
      onEditChange(false);
    },
  });

  const cancelEdit = () => {
    setDateEdit(new Date(transaction.date));
    setBudgetIdEdit(transaction.budget_id);
    setAmountEdit(transaction.amount);
    setPayeeIdEdit(transaction.payee_id);
    onEditChange(false);
  };

  useEffect(() => {
    if (!isEdit) {
      cancelEdit();
    }
  }, [isEdit]);

  return (
    <>
      <Table.Tr>
        <Table.Td>
          <Checkbox
            checked={selected}
            onChange={(x) => onCheckboxSelectedChange(x.currentTarget.checked)}
          />
        </Table.Td>
        {!isEdit && (
          <Table.Td onClick={() => !selected && onRowSelected()}>
            <span onClick={() => onEditChange(true)}>
              {new Date(transaction.date).toDateString()}
            </span>
          </Table.Td>
        )}
        {isEdit && (
          <Table.Td>
            <DatePickerInput
              value={dateEdit}
              onChange={(x) => x && setDateEdit(x)}
            />
          </Table.Td>
        )}
        {!isEdit && (
          <Table.Td onClick={() => !selected && onRowSelected()}>
            <span onClick={() => onEditChange(true)}>
              {budgetsMap.get(transaction.budget_id)?.name}
            </span>
          </Table.Td>
        )}
        {isEdit && (
          <Table.Td>
            <Select
              value={budgetIdEdit}
              data={Array.from(budgetsMap.values()).map((x) => ({
                label: x.name,
                value: x.id,
              }))}
              onChange={(x) => x && setBudgetIdEdit(x)}
            />
          </Table.Td>
        )}
        {!isEdit && (
          <Table.Td onClick={() => !selected && onRowSelected()}>
            <span onClick={() => onEditChange(true)}>
              {Math.sign(transaction.amount) === -1 && "-"}$
              {Math.abs(transaction.amount).toFixed(2)}
            </span>
          </Table.Td>
        )}
        {isEdit && (
          <Table.Td>
            <NumberInput
              value={amountEdit}
              onChange={(x) => typeof x === "number" && setAmountEdit(x)}
            />
          </Table.Td>
        )}
        {!isEdit && (
          <Table.Td onClick={() => !selected && onRowSelected()}>
            <span onClick={() => onEditChange(true)}>
              {payeesMap.get(transaction.payee_id)?.name}
            </span>
          </Table.Td>
        )}
        {isEdit && (
          <Table.Td>
            <Select
              value={payeeIdEdit}
              onChange={(x) => x && setPayeeIdEdit(x)}
              data={Array.from(payeesMap.values()).map((x) => ({
                value: x.id,
                label: x.name,
              }))}
            />
          </Table.Td>
        )}
      </Table.Tr>

      {isEdit && (
        <Table.Tr>
          <Table.Td>
            <Button onClick={cancelEdit}>Cancel</Button>
          </Table.Td>
          <Table.Td>
            <Button onClick={() => saveTransaction.mutate()}>Save</Button>
          </Table.Td>
        </Table.Tr>
      )}
    </>
  );
}
