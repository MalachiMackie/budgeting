import { Title } from "@mantine/core";
import {
  QueryClient,
  queryOptions,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { Params, useParams } from "react-router-dom";
import { BudgetingApi } from "../api/budgetingApi";
import { useBudgetingApi } from "../App";
import { useUserId } from "../hooks/useUserId";
import { queryKeys } from "../queryKeys";
import { TransactionList } from "../views/TransactionList";

export function AccountPage(): JSX.Element {
  const userId = useUserId();
  const api = useBudgetingApi();
  const params = useParams();

  const { data: bankAccount } = useSuspenseQuery(
    createQueryOptions(api, params.accountId!, userId)
  );

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

function createQueryOptions(
  api: BudgetingApi,
  accountId: string,
  userId: string
) {
  return queryOptions({
    queryKey: queryKeys.bankAccounts.fetchSingle(accountId),
    queryFn: () => api.getBankAccount(accountId, userId),
  });
}

export function createAccountLoader(
  api: BudgetingApi,
  queryClient: QueryClient,
  userId: string
) {
  return async ({ params }: { params: Params }) => {
    const accountId = params.accountId!;
    await queryClient.ensureQueryData(
      createQueryOptions(api, accountId, userId)
    );
    return null;
  };
}
