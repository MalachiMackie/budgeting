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
import { PayeesList } from "../views/PayeesList";

export function PayeesPage(): JSX.Element {
  const api = useBudgetingApi();
  const userId = useUserId();

  const {
    data: { data: payees },
  } = useSuspenseQuery(createQueryOptions(api, userId));

  return (
    <div>
      <Title>Payees</Title>
      <PayeesList payees={payees} />
    </div>
  );
}

function createQueryOptions(api: Client, userId: string) {
  return queryOptions({
    queryKey: queryKeys.payees.fetch(userId),
    queryFn: () => api.getPayees({ user_id: userId }),
  });
}

export function createPayeesLoader(
  api: Client,
  queryClient: QueryClient,
  userId: string
) {
  return async () => {
    await queryClient.ensureQueryData(createQueryOptions(api, userId));
    return null;
  };
}
