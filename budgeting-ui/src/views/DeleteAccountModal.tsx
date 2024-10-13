import { Button, Flex, Modal } from "@mantine/core";
import { useMutation } from "@tanstack/react-query";
import { BankAccount } from "../api/budgetingApi";
import { useBudgetingApi } from "../App";
import { useUserId } from "../hooks/useUserId";
import { queryKeys } from "../queryKeys";

export type DeleteAccountModalProps = {
  bankAccount: BankAccount;
  onDelete: () => void;
  onCancel: () => void;
};

export function DeleteAccountModal({
  bankAccount: { name, id },
  onDelete,
  onCancel,
}: DeleteAccountModalProps): JSX.Element {
  const api = useBudgetingApi();
  const userId = useUserId();

  const deleteAccount = useMutation({
    mutationKey: queryKeys.bankAccounts.delete(id),
    mutationFn: async () => {
      await api.deleteBankAccount(id, userId);
    },
    onSuccess: async () => {
      onDelete();
    },
  });

  return (
    <Modal opened onClose={onCancel} title="Delete Account">
      <span>Are you sure you want to delete Bank Account "{name}"?</span>
      <Flex gap={"0.5rem"} mt={"1rem"} justify={"flex-end"}>
        <Button variant="subtle" onClick={onCancel}>
          Cancel
        </Button>
        <Button color="red" onClick={() => deleteAccount.mutate()}>
          Delete
        </Button>
      </Flex>
    </Modal>
  );
}
