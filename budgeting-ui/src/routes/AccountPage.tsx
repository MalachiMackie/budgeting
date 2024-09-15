import { Title } from "@mantine/core";
import { Params, useLoaderData } from "react-router-dom";
import { BankAccount, BudgetingApi } from "../api/budgetingApi";
import { useUserId } from "../hooks/useUserId";
import TransactionList from "../views/transactionList";

export function AccountPage(): JSX.Element {
  const bankAccount = useLoaderData() as BankAccount;
  const userId = useUserId();

  return (
    <>
      <Title>{bankAccount.name}</Title>
      <TransactionList bankAccountId={bankAccount.id} userId={userId} />
    </>
  );
}

export function createAccountLoader(api: BudgetingApi, userId: string) {
  return ({ params }: { params: Params }) => {
    return api.getBankAccount(params.accountId!, userId);
  };
}
