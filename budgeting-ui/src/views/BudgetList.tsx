import { Checkbox, Table } from "@mantine/core";
import { Budget } from "../api/client";

export type BudgetListProps = {
  budgets: Budget[];
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
          <Table.Th>Assigned</Table.Th>
          <Table.Th>Activity</Table.Th>
          <Table.Th>Available</Table.Th>
        </Table.Tr>
      </Table.Thead>
      <Table.Tbody>
        {budgets.map((b) => (
          <BudgetRow
            key={b.id}
            selected={selectedId === b.id}
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
}: {
  budget: Budget;
  selected: boolean;
  onSelectedChange: (selected: boolean) => void;
}): JSX.Element {
  return (
    <Table.Tr>
      <Table.Td>
        <Checkbox
          checked={selected}
          onChange={(e) => onSelectedChange(e.currentTarget.checked)}
        />
      </Table.Td>
      <Table.Td>{budget.name}</Table.Td>
      <Table.Td></Table.Td>
      <Table.Td></Table.Td>
      <Table.Td></Table.Td>
    </Table.Tr>
  );
}
