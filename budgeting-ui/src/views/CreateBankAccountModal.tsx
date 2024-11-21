import { Button, Flex, Modal, NumberInput, TextInput } from "@mantine/core";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useBudgetingApi } from "../App";
import { useUserId } from "../hooks/useUserId";
import { queryKeys } from "../queryKeys";

export type CreateBankAccountModalProps = {
  onClose: () => void;
  onSuccess: () => void;
};

export function CreateBankAccountModal({
  onClose,
  onSuccess,
}: CreateBankAccountModalProps): JSX.Element {
  const [name, setName] = useState("");
  const [initialBalance, setInitialBalance] = useState(0);

  const userId = useUserId();
  const api = useBudgetingApi();
  const queryClient = useQueryClient();

  const create = useMutation({
    mutationKey: queryKeys.bankAccounts.create,
    mutationFn: () =>
      api.createBankAccount(
        {},
        {
          initial_amount: initialBalance,
          name: name,
          user_id: userId,
        }
      ),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.bankAccounts.fetch,
      });
      onSuccess();
    },
  });

  // todo: loading

  return (
    <Modal
      withCloseButton
      onClose={onClose}
      opened={true}
      title="Create Bank Account"
    >
      <TextInput
        value={name}
        onChange={(e) => setName(e.currentTarget.value)}
        label="Name"
      />
      <NumberInput
        value={initialBalance}
        onChange={(e) => typeof e === "number" && setInitialBalance(e)}
        label="Initial Balance"
      />
      <Flex gap={"0.5rem"} mt={"1rem"} justify={"flex-end"}>
        <Button variant="subtle" onClick={onClose}>
          Cancel
        </Button>
        <Button onClick={() => create.mutate()}>Create</Button>
      </Flex>
    </Modal>
  );
}
