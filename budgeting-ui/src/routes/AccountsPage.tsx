import { Title } from "@mantine/core";
import { useLoaderData } from "react-router-dom";
import { BankAccount, BudgetingApi } from "../api/budgetingApi";
import { useUserId } from "../hooks/useUserId";
import { BankAccountList } from "../views/BankAccountList";

export function AccountsPage(): JSX.Element {
  const userId = useUserId();
  const bankAccounts = useLoaderData() as BankAccount[];

  return (
    <div>
      <Title size="h1">Bank Accounts</Title>
      <BankAccountList userId={userId} bankAccounts={bankAccounts} />
    </div>
  );
}

export function createAccountsLoader(api: BudgetingApi, userId: string) {
  return () => {
    return api.getBankAccounts(userId);
  };
}
