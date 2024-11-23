import { Button, Flex, Modal } from "@mantine/core";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Payee } from "../api/client";
import { useBudgetingApi } from "../App";
import { useUserId } from "../hooks/useUserId";
import { queryKeys } from "../queryKeys";

export type DeletePayeeModalProps = {
  payee: Payee;
  onSuccess: () => void;
  onCancel: () => void;
};

export function DeletePayeeModal({
  payee,
  onCancel,
  onSuccess,
}: DeletePayeeModalProps): JSX.Element {
  const budgetingApi = useBudgetingApi();
  const queryClient = useQueryClient();
  const userId = useUserId();

  const deletePayee = useMutation({
    mutationKey: queryKeys.payees.delete,
    mutationFn: ({ payeeId }: { payeeId: string }) =>
      budgetingApi.deletePayee({ payeeId: payeeId }),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({
          queryKey: queryKeys.payees.fetch(userId),
        }),
      ]);
      onSuccess();
    },
  });

  return (
    <Modal opened onClose={onCancel} title="Delete Account">
      <span>Are you sure you want to delete payee "{payee.name}"?</span>
      <Flex gap={"0.5rem"} mt={"1rem"} justify={"flex-end"}>
        <Button variant="subtle" onClick={onCancel}>
          Cancel
        </Button>
        <Button
          color="red"
          onClick={() => deletePayee.mutate({ payeeId: payee.id })}
        >
          Delete
        </Button>
      </Flex>
    </Modal>
  );
}
