import { Title } from "@mantine/core";
import { QueryClient } from "@tanstack/react-query";
import { useLoaderData } from "react-router-dom";
import { Budget, BudgetingApi } from "../api/budgetingApi";
import { BudgetList } from "../views/BudgetList";

export function BudgetsPage(): JSX.Element {
  const budgets = useLoaderData() as Budget[];

  return (
    <div>
      <Title size="h1">Budgets</Title>
      <BudgetList budgets={budgets} />
    </div>
  );
}

export function createBudgetsLoader(
  api: BudgetingApi,
  queryClient: QueryClient,
  userId: string
) {
  return () => {
    return queryClient.fetchQuery({
      queryKey: queryKeys.budgets.fetch,
      queryFn: () => api.getBudgets(userId),
    });
  };
}
