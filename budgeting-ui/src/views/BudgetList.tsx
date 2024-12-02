import {
  Button,
  Checkbox,
  ComboboxItem,
  Flex,
  Menu,
  Modal,
  NumberInput,
  Select,
  Table,
} from "@mantine/core";
import { IconMenu, IconSwitchVertical } from "@tabler/icons-react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { GetBudgetResponse } from "../api/client";
import { useBudgetingApi } from "../App";
import { queryKeys } from "../queryKeys";
import { formatCurrency } from "../utils/formatCurrency";
import { formatDateForApi } from "../utils/formatDate";

export type BudgetListProps = {
  budgets: GetBudgetResponse[];
  selectedId: string | null;
  onSelectedChange: (selectedId: string | null) => void;
};

export function BudgetList({
  budgets,
  selectedId,
  onSelectedChange,
}: BudgetListProps): JSX.Element {
  return (
    <Table>
      <Table.Thead>
        <Table.Tr>
          <Table.Th />
          <Table.Th>Name</Table.Th>
          <Table.Th>Spent</Table.Th>
          <Table.Th>Assigned</Table.Th>
          <Table.Th />
        </Table.Tr>
      </Table.Thead>
      <Table.Tbody>
        {budgets.map((b) => (
          <BudgetRow
            key={b.id}
            selected={selectedId === b.id}
            budgets={budgets}
            onSelectedChange={(selected) =>
              onSelectedChange(selected ? b.id : null)
            }
            budget={b}
          />
        ))}
      </Table.Tbody>
    </Table>
  );
}

function BudgetRow({
  budget,
  selected,
  onSelectedChange,
  budgets,
}: {
  budget: GetBudgetResponse;
  selected: boolean;
  onSelectedChange: (selected: boolean) => void;
  budgets: GetBudgetResponse[];
}): JSX.Element {
  const [showMoveMoneyModal, setShowMoveMoneyModal] = useState(false);

  return (
    <>
      <Table.Tr>
        <Table.Td>
          <Checkbox
            checked={selected}
            onChange={(e) => onSelectedChange(e.currentTarget.checked)}
          />
        </Table.Td>
        <Table.Td>{budget.name}</Table.Td>
        <Table.Td>
          {formatCurrency(
            budget.assignments
              .filter((x) => x.source.type === "Transaction" && x.amount <= 0)
              .map((x) => x.amount)
              .reduce((prev, current) => prev + current, 0)
          )}
        </Table.Td>
        <Table.Td>{formatCurrency(budget.total_assigned)}</Table.Td>
        <Table.Td>
          <Menu>
            <Menu.Target>
              <Button variant="default">
                <IconMenu />
              </Button>
            </Menu.Target>
            <Menu.Dropdown>
              <Menu.Item onClick={() => setShowMoveMoneyModal(true)}>
                Move
              </Menu.Item>
            </Menu.Dropdown>
          </Menu>
        </Table.Td>
      </Table.Tr>
      {showMoveMoneyModal && (
        <MoveMoneyModal
          fromBudgetId={budget.id}
          toBudgetId={null}
          budgets={budgets}
          onFinished={() => setShowMoveMoneyModal(false)}
        />
      )}
    </>
  );
}

type MoveMoneyModalProps = {
  onFinished: () => void;
  budgets: GetBudgetResponse[];
  fromBudgetId: string | null;
  toBudgetId: string | null;
};

function MoveMoneyModal({
  onFinished,
  budgets,
  fromBudgetId: initialFromBudgetId,
  toBudgetId: initialToBudgetId,
}: MoveMoneyModalProps): JSX.Element {
  const [fromBudgetId, setFromBudgetId] = useState(initialFromBudgetId);
  const [toBudgetId, setToBudgetId] = useState(initialToBudgetId);
  const [amount, setAmount] = useState<number | null>(null);

  const fromBudgetOptions: ComboboxItem[] = budgets.map((x) => ({
    label: `${x.name} (${formatCurrency(x.total_assigned)})`,
    value: x.id.toString(),
    disabled: x.id.toString() === toBudgetId,
  }));
  const toBudgetOptions: ComboboxItem[] = budgets.map((x) => ({
    label: `${x.name} (${formatCurrency(x.total_assigned)})`,
    value: x.id.toString(),
    disabled: x.id.toString() === fromBudgetId,
  }));

  const api = useBudgetingApi();
  const queryClient = useQueryClient();

  const save = useMutation({
    mutationKey: queryKeys.budgets.moveBetween(
      fromBudgetId ?? "",
      toBudgetId ?? ""
    ),
    mutationFn: async (amount: number) => {
      if (fromBudgetId === null || toBudgetId === null) {
        throw new Error("Cannot move between budgets without both selected");
      }
      await api.transferBetweenBudgets(
        {
          budgetId: fromBudgetId,
          otherBudgetId: toBudgetId,
        },
        { date: formatDateForApi(new Date()), amount }
      );
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.budgets.fetch,
      });
      onFinished();
    },
  });

  return (
    <Modal title="Move between budgets" opened={true} onClose={onFinished}>
      <Select
        label="From"
        onChange={setFromBudgetId}
        value={fromBudgetId}
        data={fromBudgetOptions}
      />
      <Button
        m={"sm"}
        onClick={() => {
          setFromBudgetId(toBudgetId);
          setToBudgetId(fromBudgetId);
        }}
        disabled={fromBudgetId === null || toBudgetId === null}
      >
        <IconSwitchVertical />
      </Button>
      <Select
        label="To"
        onChange={setToBudgetId}
        value={toBudgetId}
        data={toBudgetOptions}
      />
      <NumberInput
        label="Amount"
        value={amount ?? undefined}
        onChange={(x) => typeof x === "number" && setAmount(x)}
      />
      <Flex mt={"sm"} justify={"flex-end"} gap={"xs"}>
        <Button onClick={() => onFinished()}>Cancel</Button>
        <Button
          onClick={() => save.mutate(amount!)}
          disabled={amount === null || amount <= 0}
        >
          Save
        </Button>
      </Flex>
    </Modal>
  );
}
