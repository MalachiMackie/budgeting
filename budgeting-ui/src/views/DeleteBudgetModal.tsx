import { Button, Flex, Modal } from "@mantine/core";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Budget } from "../api/client";
import { useBudgetingApi } from "../App";
import { queryKeys } from "../queryKeys";

export type DeleteBudgetModalProps = {
  budget: Budget;
  onSuccess: () => void;
  onCancel: () => void;
};

export function DeleteBudgetModal({
  budget,
  onCancel,
  onSuccess,
}: DeleteBudgetModalProps): JSX.Element {
  const budgetingApi = useBudgetingApi();
  const queryClient = useQueryClient();

  const deleteBudget = useMutation({
    mutationKey: queryKeys.budgets.delete,
    mutationFn: ({ budgetId }: { budgetId: string }) =>
      budgetingApi.deleteBudget({ budget_id: budgetId }),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({
          queryKey: queryKeys.budgets.fetch,
        }),
      ]);
      onSuccess();
    },
  });

  return (
    <Modal opened onClose={onCancel} title="Delete Account">
      <span>Are you sure you want to delete budget "{budget.name}"?</span>
      <Flex gap={"0.5rem"} mt={"1rem"} justify={"flex-end"}>
        <Button variant="subtle" onClick={onCancel}>
          Cancel
        </Button>
        <Button
          color="red"
          onClick={() => deleteBudget.mutate({ budgetId: budget.id })}
        >
          Delete
        </Button>
      </Flex>
    </Modal>
  );
}
