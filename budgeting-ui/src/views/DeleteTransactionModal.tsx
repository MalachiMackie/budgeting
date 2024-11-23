import { Button, Flex, Modal } from "@mantine/core";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useBudgetingApi } from "../App";
import { queryKeys } from "../queryKeys";

export type DeleteTransactionModalProps = {
  transactionId: string;
  bankAccountId: string;
  onSuccess: () => void;
  onCancel: () => void;
};

export function DeleteTransactionModal({
  transactionId,
  bankAccountId,
  onCancel,
  onSuccess,
}: DeleteTransactionModalProps): JSX.Element {
  const budgetingApi = useBudgetingApi();
  const queryClient = useQueryClient();

  const deleteTransaction = useMutation({
    mutationKey: queryKeys.transactions.delete,
    mutationFn: ({ transactionId }: { transactionId: string }) =>
      budgetingApi.deleteTransaction({ transactionId: transactionId }),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({
          queryKey: queryKeys.transactions.fetch(bankAccountId),
        }),
        queryClient.invalidateQueries({
          queryKey: queryKeys.bankAccounts.fetch,
        }),
        queryClient.invalidateQueries({
          queryKey: queryKeys.bankAccounts.fetchSingle(bankAccountId),
        }),
      ]);
      onSuccess();
    },
  });

  return (
    <Modal opened onClose={onCancel} title="Delete Account">
      <span>Are you sure you want to delete transaction?</span>
      <Flex gap={"0.5rem"} mt={"1rem"} justify={"flex-end"}>
        <Button variant="subtle" onClick={onCancel}>
          Cancel
        </Button>
        <Button
          color="red"
          onClick={() => deleteTransaction.mutate({ transactionId })}
        >
          Delete
        </Button>
      </Flex>
    </Modal>
  );
}
