import { Button, Flex, Modal, TextInput } from "@mantine/core";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { BankAccount } from "../api/budgetingApi";
import { useBudgetingApi } from "../App";
import { useUserId } from "../hooks/useUserId";
import { queryKeys } from "../queryKeys";

export type EditBankAccountModalProps = {
  bankAccount: BankAccount;
  onCancel: () => void;
  onSuccess: () => void;
};

export function EditBankAccountModal({
  bankAccount,
  onCancel,
  onSuccess,
}: EditBankAccountModalProps): JSX.Element {
  const [accountName, setAccountName] = useState(bankAccount.name);

  const api = useBudgetingApi();
  const queryClient = useQueryClient();
  const userId = useUserId();

  const saveBankAccount = useMutation({
    mutationKey: queryKeys.bankAccounts.edit(bankAccount.id),
    mutationFn: () =>
      api.updateBankAccount(bankAccount.id, userId, { name: accountName }),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({
          queryKey: queryKeys.bankAccounts.fetch,
        }),
        queryClient.invalidateQueries({
          queryKey: queryKeys.bankAccounts.fetchSingle(bankAccount.id),
        }),
      ]);
      onSuccess();
    },
  });

  return (
    <Modal opened onClose={onCancel} title="Edit Bank Account">
      <Flex direction={"column"} gap={"xs"}>
        <TextInput
          label="Account name"
          value={accountName}
          onChange={(e) => setAccountName(e.currentTarget.value)}
        />
        <Button onClick={() => saveBankAccount.mutate()}>Save</Button>
      </Flex>
    </Modal>
  );
}
