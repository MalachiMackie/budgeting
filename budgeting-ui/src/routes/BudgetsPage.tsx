import { Button, Flex, Title } from "@mantine/core";
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
import { EditBudgetModal } from "../views/EditBudgetModal";

export function BudgetsPage(): JSX.Element {
  const api = useBudgetingApi();
  const userId = useUserId();
  const { data: budgets } = useSuspenseQuery(createQueryOptions(api, userId));

  const [selectedBudgetId, setSelectedBudgetId] = useState<string | null>(null);
  const [showCreateBudget, setShowCreateBudget] = useState(false);
  const [showUpdateBudget, setShowUpdateBudget] = useState(false);

  return (
    <div>
      <Title mb="1rem" size="h1">
        Budgets
      </Title>
      <Flex gap={"xs"}>
        <Button onClick={() => setShowCreateBudget(true)}>Create Budget</Button>
        <Button
          onClick={() => setShowUpdateBudget(true)}
          disabled={!selectedBudgetId}
        >
          Update Budget
        </Button>
      </Flex>
      <BudgetList
        selectedId={selectedBudgetId}
        onSelectedChange={setSelectedBudgetId}
        budgets={budgets}
      />
      {showCreateBudget && (
        <CreateBudgetModal
          onCancel={() => setShowCreateBudget(false)}
          onSuccess={() => {
            setShowCreateBudget(false);
          }}
        />
      )}
      {showUpdateBudget && (
        <EditBudgetModal
          budget={budgets.find((x) => x.id === selectedBudgetId)!}
          onCancel={() => setShowUpdateBudget(false)}
          onSuccess={() => setShowUpdateBudget(false)}
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
