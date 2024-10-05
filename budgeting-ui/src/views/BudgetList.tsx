import { Table } from "@mantine/core";
import { Budget } from "../api/budgetingApi";

export type BudgetListProps = {
  budgets: Budget[];
};

export function BudgetList({ budgets }: BudgetListProps): JSX.Element {
  return (
    <Table>
      <Table.Thead>
        <Table.Tr>
          <Table.Th>Name</Table.Th>
          <Table.Th>Assigned</Table.Th>
          <Table.Th>Activity</Table.Th>
          <Table.Th>Available</Table.Th>
        </Table.Tr>
      </Table.Thead>
      <Table.Tbody>
        {budgets.map((b) => (
          <BudgetRow key={b.id} budget={b} />
        ))}
      </Table.Tbody>
    </Table>
  );
}

function BudgetRow({ budget }: { budget: Budget }): JSX.Element {
  return (
    <Table.Tr>
      <Table.Td>{budget.name}</Table.Td>
      <Table.Td></Table.Td>
      <Table.Td></Table.Td>
      <Table.Td></Table.Td>
    </Table.Tr>
  );
}
