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
import "./NewTransactionRow.css";

export function NewTransactionRow({
  payees,
  save,
}: {
  payees: Payee[];
  save: (x: CreateTransactionRequest) => Promise<void>;
}): JSX.Element {
  // todo: show loading
  const [date, setDate] = useState(new Date());
  const [amount, setAmount] = useState<number | undefined>(undefined);
  const [payeeId, setPayeeId] = useState<string | undefined>(undefined);
  const [saving, setSaving] = useState(false);
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
            onChange={(x) => x && setPayeeId(x)}
            data={payees.map((x) => ({ value: x.id, label: x.name }))}
            value={payeeId}
          />
        </Table.Td>
      </Table.Tr>
      <Table.Tr>
        <Table.Td colSpan={3}>
          <Button onClick={handleSaveClick}>Save</Button>
        </Table.Td>
      </Table.Tr>
    </>
  );
}

function formatDate(date: Date): string {
  let sb: (string | number)[] = [date.getFullYear(), "-"];
  const month = date.getMonth() + 1;
  if (month < 10) {
    sb.push("0");
  }
  sb.push(month, "-");

  const dayOfMonth = date.getDate();
  if (dayOfMonth < 10) {
    sb.push("0");
  }
  sb.push(dayOfMonth);

  return sb.join("");
}
