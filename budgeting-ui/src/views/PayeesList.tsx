import { Button, Table } from "@mantine/core";
import { IconPencil, IconTrash } from "@tabler/icons-react";
import { useState } from "react";
import { Payee } from "../api/client";
import { DeletePayeeModal } from "./DeletePayeeModal";
import { EditPayeeModal } from "./EditPayeeModal";

export type PayeesListProps = { payees: Payee[] };

export function PayeesList({ payees }: PayeesListProps): JSX.Element {
  const [editPayee, setEditPayee] = useState<Payee | null>(null);
  const [deletePayee, setDeletePayee] = useState<Payee | null>(null);

  return (
    <>
      <Table>
        <Table.Thead>
          <Table.Tr>
            <Table.Th>Name</Table.Th>
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {payees.map((x) => (
            <Table.Tr key={x.id}>
              <Table.Td>{x.name}</Table.Td>
              <Table.Td>
                <Button onClick={() => setEditPayee(x)}>
                  <IconPencil />
                </Button>
              </Table.Td>
              <Table.Td>
                <Button onClick={() => setDeletePayee(x)}>
                  <IconTrash color="red" />
                </Button>
              </Table.Td>
            </Table.Tr>
          ))}
        </Table.Tbody>
      </Table>
      {editPayee && (
        <EditPayeeModal
          payee={editPayee}
          onCancel={() => setEditPayee(null)}
          onSuccess={() => setEditPayee(null)}
        />
      )}
      {deletePayee && (
        <DeletePayeeModal
          payee={deletePayee}
          onCancel={() => setDeletePayee(null)}
          onSuccess={() => setDeletePayee(null)}
        />
      )}
    </>
  );
}
