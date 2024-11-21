import { Button, Flex, Modal, TextInput } from "@mantine/core";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Payee } from "../api/client";
import { useBudgetingApi } from "../App";
import { useUserId } from "../hooks/useUserId";
import { queryKeys } from "../queryKeys";

export type CreatePayeeModalProps = {
  onCancel: () => void;
  onSuccess: (newPayee: Payee) => void;
};

export function CreatePayeeModal({
  onCancel,
  onSuccess,
}: CreatePayeeModalProps): JSX.Element {
  const [name, setName] = useState("");
  const trimmed = name.trim();

  const api = useBudgetingApi();
  const userId = useUserId();
  const queryClient = useQueryClient();

  const createPayee = useMutation({
    mutationKey: queryKeys.payees.create,
    mutationFn: () =>
      api.createPayee(
        {},
        {
          user_id: userId,
          name: trimmed,
        }
      ),
    onSuccess: (payeeId) => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.payees.fetch(userId),
      });
      onSuccess({ name: trimmed, id: payeeId.data, user_id: userId });
    },
  });

  const isValid = trimmed.length > 0;

  return (
    <Modal title="Create Payee" onClose={onCancel} opened={true}>
      <TextInput
        value={name}
        onChange={(e) => setName(e.currentTarget.value)}
      />
      <Flex gap={"0.5rem"} mt={"1rem"} justify={"flex-end"}>
        <Button variant="subtle" onClick={onCancel}>
          Cancel
        </Button>
        <Button onClick={() => createPayee.mutate()} disabled={!isValid}>
          Create
        </Button>
      </Flex>
    </Modal>
  );
}
