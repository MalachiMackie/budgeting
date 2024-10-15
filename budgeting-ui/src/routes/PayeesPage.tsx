import { Title } from "@mantine/core";
import {
  QueryClient,
  queryOptions,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { BudgetingApi } from "../api/budgetingApi";
import { useBudgetingApi } from "../App";
import { useUserId } from "../hooks/useUserId";
import { queryKeys } from "../queryKeys";
import { PayeesList } from "../views/PayeesList";

export function PayeesPage(): JSX.Element {
  const api = useBudgetingApi();
  const userId = useUserId();

  const { data: payees } = useSuspenseQuery(createQueryOptions(api, userId));

  return (
    <div>
      <Title>Payees</Title>
      <PayeesList payees={payees} />
    </div>
  );
}

function createQueryOptions(api: BudgetingApi, userId: string) {
  return queryOptions({
    queryKey: queryKeys.payees.fetch(userId),
    queryFn: () => api.getPayees(userId),
  });
}

export function createPayeesLoader(
  api: BudgetingApi,
  queryClient: QueryClient,
  userId: string
) {
  return async () => {
    await queryClient.ensureQueryData(createQueryOptions(api, userId));
    return null;
  };
}
