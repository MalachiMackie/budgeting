import { Title } from "@mantine/core";
import {
  QueryClient,
  queryOptions,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { Client } from "../api/client";
import { useBudgetingApi } from "../App";
import { useUserId } from "../hooks/useUserId";
import { queryKeys } from "../queryKeys";
import { BankAccountList } from "../views/BankAccountList";

export function AccountsPage(): JSX.Element {
  const api = useBudgetingApi();
  const userId = useUserId();

  const {
    data: { data: bankAccounts },
  } = useSuspenseQuery(createQueryOptions(api, userId));

  return (
    <div>
      <Title size="h1">Bank Accounts</Title>
      <BankAccountList bankAccounts={bankAccounts} />
    </div>
  );
}

function createQueryOptions(api: Client, userId: string) {
  return queryOptions({
    queryKey: queryKeys.bankAccounts.fetch,
    queryFn: () => api.getBankAccounts({ user_id: userId }),
  });
}

export function createAccountsLoader(
  api: Client,
  queryClient: QueryClient,
  userId: string
) {
  return async () => {
    await queryClient.fetchQuery(createQueryOptions(api, userId));
    return null;
  };
}
