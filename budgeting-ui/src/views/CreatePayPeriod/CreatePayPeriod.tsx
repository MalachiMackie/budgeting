import { Select } from "@mantine/core";
import { DatePickerInput } from "@mantine/dates";
import { useState } from "react";

type PayPeriod =
  | {
      type: "monthly";
      payDay: { type: "DateOfMonth"; date: DayOfWeek };
    }
  | {
      type: "fortnightly";
      examplePayDay: Date;
    }
  | { type: "weekly"; dayOfWeek: DayOfWeek };

const daysOfWeek = [
  "Monday",
  "Tuesday",
  "Wednesday",
  "Thursday",
  "Friday",
  "Saturday",
  "Sunday",
] as const;

type DayOfWeek = (typeof daysOfWeek)[number];

type DateOfMonth =
  | 1
  | 2
  | 3
  | 4
  | 5
  | 6
  | 7
  | 8
  | 9
  | 10
  | 11
  | 12
  | 13
  | 14
  | 15
  | 16
  | 17
  | 18
  | 19
  | 20
  | 21
  | 22
  | 23
  | 24
  | 25
  | 26
  | 27
  | 28;

export function CreatePayPeriod(): JSX.Element {
  const [type, setType] = useState<PayPeriod["type"]>("monthly");
  const [dateOfMonth, setDateOfMonth] = useState<DateOfMonth>(1);
  const [lastPayDay, setLastPayDay] = useState<Date>(new Date());
  const [dayOfWeek, setDayOfWeek] = useState<DayOfWeek>("Monday");

  return (
    <div>
      <Select
        label="Period Type"
        value={type}
        onChange={(x) => setType(x as typeof type)}
        data={[
          { value: "monthly", label: "Monthly" },
          { value: "fortnightly", label: "Fortnightly" },
          { value: "weekly", label: "Weekly" },
        ]}
      />
      {type === "monthly" && (
        <Select
          label={"Day of Month"}
          value={dateOfMonth.toString()}
          onChange={(x) => setDateOfMonth(parseInt(x!) as DateOfMonth)}
          data={Array.from(new Array(28).keys())
            .map((x) => x + 1)
            .map((x) => ({ value: x.toString(), label: x.toString() }))}
        />
      )}
      {type === "fortnightly" && (
        <DatePickerInput
          label={"Last Pay Day"}
          value={lastPayDay}
          onChange={(x) => setLastPayDay(x!)}
        />
      )}
      {type === "weekly" && (
        <Select
          label="Day of Week"
          value={dayOfWeek}
          onChange={(x) => setDayOfWeek(x as DayOfWeek)}
          data={daysOfWeek.map((x) => ({ value: x, label: x }))}
        />
      )}
    </div>
  );
}
