import { Title } from "@mantine/core";
import { QueryClient } from "@tanstack/react-query";
import { useLoaderData } from "react-router-dom";
import { BankAccount, BudgetingApi } from "../api/budgetingApi";
import { BankAccountList } from "../views/BankAccountList";

export function AccountsPage(): JSX.Element {
  const bankAccounts = useLoaderData() as BankAccount[];

  return (
    <div>
      <Title size="h1">Bank Accounts</Title>
      <BankAccountList bankAccounts={bankAccounts} />
    </div>
  );
}

export function createAccountsLoader(
  api: BudgetingApi,
  queryClient: QueryClient,
  userId: string
) {
  return () => {
    return queryClient.fetchQuery({
      queryKey: ["bank-accounts"],
      queryFn: () => api.getBankAccounts(userId),
    });
  };
}
