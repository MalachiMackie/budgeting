import { Button, Title } from "@mantine/core";
import { QueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useLoaderData, useRevalidator } from "react-router-dom";
import { Budget, BudgetingApi } from "../api/budgetingApi";
import { queryKeys } from "../queryKeys";
import { BudgetList } from "../views/BudgetList";
import { CreateBudgetModal } from "../views/CreateBudgetModal";

export function BudgetsPage(): JSX.Element {
  const budgets = useLoaderData() as Budget[];
  const { revalidate } = useRevalidator();

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
            revalidate();
          }}
        />
      )}
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
