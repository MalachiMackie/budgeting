import { Title } from "@mantine/core";
import { QueryClient } from "@tanstack/react-query";
import { Params, useLoaderData } from "react-router-dom";
import { BankAccount, BudgetingApi } from "../api/budgetingApi";
import { useUserId } from "../hooks/useUserId";
import { queryKeys } from "../queryKeys";
import { TransactionList } from "../views/TransactionList";

export function AccountPage(): JSX.Element {
  const bankAccount = useLoaderData() as BankAccount;
  const userId = useUserId();

  return (
    <>
      <Title>{bankAccount.name} - Transactions</Title>
      <span>
        Balance: {Math.sign(bankAccount.balance) === -1 && "-"}$
        {Math.abs(bankAccount.balance).toFixed(2)}
      </span>
      <TransactionList bankAccountId={bankAccount.id} userId={userId} />
    </>
  );
}

export function createAccountLoader(
  api: BudgetingApi,
  queryClient: QueryClient,
  userId: string
) {
  return ({ params }: { params: Params }) => {
    const accountId = params.accountId!;
    return queryClient.fetchQuery({
      queryKey: queryKeys.bankAccounts.fetchSingle(accountId),
      queryFn: () => api.getBankAccount(accountId, userId),
    });
  };
}
