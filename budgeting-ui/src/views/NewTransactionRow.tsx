import {
  Button,
  LoadingOverlay,
  NumberInput,
  Select,
  Table,
} from "@mantine/core";
import { DatePickerInput } from "@mantine/dates";
import { useState } from "react";
import { CreateTransactionRequest, Payee } from "../api/budgetingApi";
import { formatDate } from "../utils/formatDate";
import { CreatePayeeModal } from "./CreatePayeeModal";
import "./NewTransactionRow.css";

export function NewTransactionRow({
  payees,
  save,
}: {
  payees: Payee[];
  save: (x: CreateTransactionRequest) => Promise<string>;
}): JSX.Element {
  // todo: show loading
  const [date, setDate] = useState(new Date());
  const [amount, setAmount] = useState<number | undefined>(undefined);
  const [payeeId, setPayeeId] = useState<string | undefined>(undefined);
  const [saving, setSaving] = useState(false);
  const [showCreatePayee, setShowCreatePayee] = useState(false);

  const handleSaveClick = async () => {
    if (!payeeId) {
      alert("Cannot save transaction without a payee");
      return;
    }
    if (typeof amount === "undefined") {
      alert("Cannot save transaction without a transaction amount");
      return;
    }
    try {
      setSaving(true);
      await save({
        date: formatDate(date),
        payee_id: payeeId,
        amount: amount,
      });
      setDate(new Date());
      setAmount(undefined);
      setPayeeId(undefined);
    } catch {
      // handle error
    } finally {
      setSaving(false);
    }
  };

  const handlePayeeChange = (x: string | null | undefined) => {
    if (!x) {
      return;
    }

    if (x === "create-new") {
      setShowCreatePayee(true);
      return;
    }

    setPayeeId(x);
  };

  return (
    <>
      {saving && <LoadingOverlay />}
      <Table.Tr className="newTransactionRow">
        <Table.Td>
          <DatePickerInput value={date} onChange={(x) => x && setDate(x)} />
        </Table.Td>
        <Table.Td>
          <NumberInput
            value={amount}
            onChange={(value) => typeof value === "number" && setAmount(value)}
          />
        </Table.Td>
        <Table.Td>
          <Select
            onChange={handlePayeeChange}
            data={[
              { value: "create-new", label: "+ Create Payee" },
              ...payees.map((x) => ({ value: x.id, label: x.name })),
            ]}
            value={payeeId}
          />
        </Table.Td>
      </Table.Tr>
      <Table.Tr>
        <Table.Td colSpan={3}>
          <Button onClick={handleSaveClick}>Save</Button>
        </Table.Td>
      </Table.Tr>
      {showCreatePayee && (
        <CreatePayeeModal
          onCancel={() => setShowCreatePayee(false)}
          onSuccess={(payee) => {
            setPayeeId(payee.id);
            setShowCreatePayee(false);
          }}
        />
      )}
    </>
  );
}
