import { Button, Title } from "@mantine/core";
import {
  QueryClient,
  queryOptions,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { useState } from "react";
import { BudgetingApi } from "../api/budgetingApi";
import { useBudgetingApi } from "../App";
import { useUserId } from "../hooks/useUserId";
import { queryKeys } from "../queryKeys";
import { BudgetList } from "../views/BudgetList";
import { CreateBudgetModal } from "../views/CreateBudgetModal";

export function BudgetsPage(): JSX.Element {
  const api = useBudgetingApi();
  const userId = useUserId();
  const { data: budgets } = useSuspenseQuery(createQueryOptions(api, userId));

  const [showCreateBudget, setShowCreateBudget] = useState(false);

  return (
    <div>
      <Title mb="1rem" size="h1">
        Budgets
      </Title>
      <Button onClick={() => setShowCreateBudget(true)}>Create Budget</Button>
      <BudgetList budgets={budgets} />
      {showCreateBudget && (
        <CreateBudgetModal
          onCancel={() => setShowCreateBudget(false)}
          onSuccess={() => {
            setShowCreateBudget(false);
          }}
        />
      )}
    </div>
  );
}

function createQueryOptions(api: BudgetingApi, userId: string) {
  return queryOptions({
    queryKey: queryKeys.budgets.fetch,
    queryFn: () => api.getBudgets(userId),
  });
}

export function createBudgetsLoader(
  api: BudgetingApi,
  queryClient: QueryClient,
  userId: string
) {
  return async () => {
    await queryClient.ensureQueryData(createQueryOptions(api, userId));
    return null;
  };
}
