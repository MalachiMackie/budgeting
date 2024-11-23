import { Button, Flex, Modal, TextInput } from "@mantine/core";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Payee } from "../api/client";
import { useBudgetingApi } from "../App";
import { useUserId } from "../hooks/useUserId";
import { queryKeys } from "../queryKeys";

export type EditPayeeModalProps = {
  payee: Payee;
  onCancel: () => void;
  onSuccess: () => void;
};

export function EditPayeeModal({
  payee,
  onCancel,
  onSuccess,
}: EditPayeeModalProps): JSX.Element {
  const [payeeName, setPayeeName] = useState(payee.name);

  const api = useBudgetingApi();
  const queryClient = useQueryClient();
  const userId = useUserId();

  const savePayee = useMutation({
    mutationKey: queryKeys.payees.edit(payee.id),
    mutationFn: () =>
      api.updatePayee({ payeeId: payee.id }, { name: payeeName }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.payees.fetch(userId),
      });
      onSuccess();
    },
  });

  return (
    <Modal opened onClose={onCancel} title="Edit Payee">
      <Flex direction={"column"} gap={"xs"}>
        <TextInput
          label="Payee name"
          value={payeeName}
          onChange={(e) => setPayeeName(e.currentTarget.value)}
        />
        <Button
          disabled={payeeName.trim().length === 0}
          onClick={() => savePayee.mutate()}
        >
          Save
        </Button>
      </Flex>
    </Modal>
  );
}
