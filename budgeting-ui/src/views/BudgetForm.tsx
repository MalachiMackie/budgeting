import {
  Checkbox,
  Flex,
  NumberInput,
  SegmentedControl,
  SegmentedControlItem,
  TextInput,
} from "@mantine/core";
import { DatePickerInput } from "@mantine/dates";
import { SchedulePeriodType } from "../api/client";

export type BudgetFormValue = {
  name: string;
  hasTarget: boolean;
  targetAmountStr: string;
  targetType: "OneTime" | "Repeating";
  budgetRepeatingType: "BuildUpTo" | "RequireRepeating";
  schedulePeriodType: SchedulePeriodType | "Custom";
  scheduleStartingOn: Date;
  customSchedulePeriodTimes: number;
  customSchedulePeriodType: SchedulePeriodType;
};

export type BudgetFormProps = {
  value: BudgetFormValue;
  onChange: (value: BudgetFormValue) => void;
};

export function defaultBudgetFormValue(): BudgetFormValue {
  return {
    name: "",
    hasTarget: false,
    targetAmountStr: "",
    targetType: "OneTime",
    budgetRepeatingType: "BuildUpTo",
    schedulePeriodType: "Weekly",
    scheduleStartingOn: new Date(),
    customSchedulePeriodType: "Weekly",
    customSchedulePeriodTimes: 1,
  };
}

export function BudgetForm({ value, onChange }: BudgetFormProps): JSX.Element {
  return (
    <Flex gap="0.5rem" direction={"column"}>
      <TextInput
        label="Name"
        value={value.name}
        onChange={(e) => onChange({ ...value, name: e.currentTarget.value })}
      />
      <Checkbox
        label="Budget Target"
        checked={value.hasTarget}
        onChange={(e) =>
          onChange({ ...value, hasTarget: e.currentTarget.checked })
        }
      />
      {value.hasTarget && (
        <>
          <TextInput
            label="Target Amount"
            value={value.targetAmountStr}
            onChange={(e) =>
              onChange({ ...value, targetAmountStr: e.currentTarget.value })
            }
          />
          <SegmentedControl
            data={["OneTime", "Repeating"]}
            value={value.targetType}
            onChange={(x) =>
              onChange({ ...value, targetType: x as typeof value.targetType })
            }
          />
        </>
      )}
      {value.targetType === "Repeating" && (
        <>
          <SegmentedControl
            data={["BuildUpTo", "RequireRepeating"]}
            value={value.budgetRepeatingType}
            onChange={(x) =>
              onChange({
                ...value,
                budgetRepeatingType: x as typeof value.budgetRepeatingType,
              })
            }
          />
          <SegmentedControl
            data={["Weekly", "Fortnightly", "Monthly", "Yearly", "Custom"]}
            value={value.schedulePeriodType}
            onChange={(x) =>
              onChange({
                ...value,
                schedulePeriodType: x as typeof value.schedulePeriodType,
              })
            }
          />
        </>
      )}
      {value.targetType === "Repeating" &&
        value.schedulePeriodType !== "Custom" && (
          <>
            <DatePickerInput
              value={value.scheduleStartingOn}
              label="Starting on"
              onChange={(x) =>
                x && onChange({ ...value, scheduleStartingOn: x })
              }
            />
          </>
        )}
      {value.targetType === "Repeating" &&
        value.schedulePeriodType === "Custom" && (
          <>
            <Flex gap="0.5rem">
              <span style={{ verticalAlign: "middle" }}>Every</span>
              <NumberInput
                value={value.customSchedulePeriodTimes}
                onChange={(x) =>
                  typeof x === "number" &&
                  onChange({ ...value, customSchedulePeriodTimes: x })
                }
              />
            </Flex>

            <SegmentedControl
              data={(
                ["Weekly", "Fortnightly", "Monthly", "Yearly"] as const
              ).map(
                (newValue) =>
                  ({
                    value: newValue,
                    label: formatPeriod(
                      value.customSchedulePeriodTimes > 1,
                      newValue
                    ),
                  }) satisfies SegmentedControlItem
              )}
              value={value.customSchedulePeriodType}
              onChange={(x) =>
                onChange({
                  ...value,
                  customSchedulePeriodType:
                    x as typeof value.customSchedulePeriodType,
                })
              }
            />
          </>
        )}
    </Flex>
  );
}

function formatPeriod(plural: boolean, period: SchedulePeriodType) {
  let singular: string;
  switch (period) {
    case "Weekly":
      singular = "Week";
      break;
    case "Fortnightly":
      singular = "Fortnight";
      break;
    case "Monthly":
      singular = "Month";
      break;
    case "Yearly":
      singular = "Year";
      break;
  }
  return plural ? singular + "s" : singular;
}
