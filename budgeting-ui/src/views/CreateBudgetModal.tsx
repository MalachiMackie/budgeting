import {
  Button,
  Checkbox,
  Flex,
  Modal,
  NumberInput,
  SegmentedControl,
  SegmentedControlItem,
  TextInput,
} from "@mantine/core";
import { DatePickerInput } from "@mantine/dates";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import {
  BudgetRepeatingType,
  CreateBudgetRequest,
  CreateBudgetTargetRequest,
  SchedulePeriod,
  SchedulePeriodType,
} from "../api/budgetingApi";
import { useBudgetingApi } from "../App";
import { useUserId } from "../hooks/useUserId";
import { queryKeys } from "../queryKeys";
import { formatDate } from "../utils/formatDate";

type CreateBudgetModalProps = {
  onCancel: () => void;
  onSuccess: () => void;
};

export function CreateBudgetModal({
  onCancel,
  onSuccess,
}: CreateBudgetModalProps): JSX.Element {
  const [name, setName] = useState("");
  const [hasTarget, setHasTarget] = useState(false);
  const [targetAmountStr, setTargetAmountStr] = useState<string>("");
  const [targetType, setTargetType] = useState<"OneTime" | "Repeating">(
    "OneTime"
  );
  const [budgetRepeatingType, setBudgetRepeatingType] =
    useState<BudgetRepeatingType>("BuildUpTo");
  const [schedulePeriodType, setSchedulePeriodType] = useState<
    SchedulePeriodType | "Custom"
  >("Weekly");
  const [scheduleStartingOn, setScheduleStartingOn] = useState<Date>(
    new Date()
  );
  const [customSchedulePeriodType, setCustomSchedulePeriodType] =
    useState<SchedulePeriodType>("Weekly");
  const [customSchedulePeriodTimes, setCustomSchedulePeriodTimes] =
    useState<number>(1);

  const api = useBudgetingApi();
  const userId = useUserId();
  const queryClient = useQueryClient();

  const request: CreateBudgetRequest = {
    name: name,
    target: buildCreateTargetRequest(
      hasTarget,
      targetType,
      targetAmountStr,
      budgetRepeatingType,
      schedulePeriodType,
      scheduleStartingOn,
      customSchedulePeriodType,
      customSchedulePeriodTimes
    ),
    user_id: userId,
  };

  // todo: error messages
  const isValid = validate(request);

  const createBudget = useMutation({
    mutationKey: queryKeys.budgets.create,
    mutationFn: () => api.createBudget(request),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.budgets.fetch,
      });
      onSuccess();
    },
  });

  return (
    <Modal opened onClose={onCancel} title="Create New Budget">
      <Flex gap="0.5rem" direction={"column"}>
        <TextInput
          label="Name"
          value={name}
          onChange={(e) => setName(e.currentTarget.value)}
        />
        <Checkbox
          label="Budget Target"
          checked={hasTarget}
          onChange={(e) => setHasTarget(e.currentTarget.checked)}
        />
        {hasTarget && (
          <>
            <TextInput
              label="Target Amount"
              value={targetAmountStr}
              onChange={(e) => setTargetAmountStr(e.currentTarget.value)}
            />
            <SegmentedControl
              data={["OneTime", "Repeating"]}
              value={targetType}
              onChange={(x) => setTargetType(x as typeof targetType)}
            />
          </>
        )}
        {targetType === "Repeating" && (
          <>
            <SegmentedControl
              data={["BuildUpTo", "RequireRepeating"]}
              value={budgetRepeatingType}
              onChange={(x) =>
                setBudgetRepeatingType(x as typeof budgetRepeatingType)
              }
            />
            <SegmentedControl
              data={["Weekly", "Fortnightly", "Monthly", "Yearly", "Custom"]}
              value={schedulePeriodType}
              onChange={(x) =>
                setSchedulePeriodType(x as typeof schedulePeriodType)
              }
            />
          </>
        )}
        {targetType === "Repeating" && schedulePeriodType !== "Custom" && (
          <>
            <DatePickerInput
              value={scheduleStartingOn}
              label="Starting on"
              onChange={(x) => x && setScheduleStartingOn(x)}
            />
          </>
        )}
        {targetType === "Repeating" && schedulePeriodType === "Custom" && (
          <>
            <Flex gap="0.5rem">
              <span style={{ verticalAlign: "middle" }}>Every</span>
              <NumberInput
                value={customSchedulePeriodTimes}
                onChange={(x) =>
                  typeof x === "number" && setCustomSchedulePeriodTimes(x)
                }
              />
            </Flex>

            <SegmentedControl
              data={(
                ["Weekly", "Fortnightly", "Monthly", "Yearly"] as const
              ).map(
                (value) =>
                  ({
                    value: value,
                    label: formatPeriod(customSchedulePeriodTimes > 1, value),
                  }) satisfies SegmentedControlItem
              )}
              value={customSchedulePeriodType}
              onChange={(x) =>
                setCustomSchedulePeriodType(
                  x as typeof customSchedulePeriodType
                )
              }
            />
          </>
        )}
      </Flex>
      <Flex gap={"0.5rem"} mt={"1rem"} justify={"flex-end"}>
        <Button variant="subtle" onClick={onCancel}>
          Cancel
        </Button>
        <Button onClick={() => createBudget.mutate()} disabled={!isValid}>
          Create
        </Button>
      </Flex>
    </Modal>
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

function buildCreateTargetRequest(
  hasTarget: boolean,
  targetType: "OneTime" | "Repeating",
  targetAmountStr: string,
  budgetRepeatingType: BudgetRepeatingType,
  schedulePeriodType: SchedulePeriodType | "Custom",
  scheduleStartingOn: Date,
  customSchedulePeriodType: SchedulePeriodType,
  customSchedulePeriodTimes: number
): CreateBudgetTargetRequest | null {
  if (!hasTarget) {
    return null;
  }

  if (targetType === "OneTime") {
    return {
      OneTime: {
        target_amount: parseFloat(targetAmountStr),
      },
    };
  }

  const startingOnStr = formatDate(scheduleStartingOn);

  let schedulePeriod: SchedulePeriod;

  switch (schedulePeriodType) {
    case "Weekly":
      schedulePeriod = { Weekly: { starting_on: startingOnStr } };
      break;
    case "Fortnightly":
      schedulePeriod = { Fortnightly: { starting_on: startingOnStr } };
      break;
    case "Monthly":
      schedulePeriod = { Monthly: { starting_on: startingOnStr } };
      break;
    case "Yearly":
      schedulePeriod = { Yearly: { starting_on: startingOnStr } };
      break;
    case "Custom":
      schedulePeriod = {
        Custom: {
          every_x_periods: customSchedulePeriodTimes,
          period: customSchedulePeriodType,
        },
      };
      break;
  }

  return {
    Repeating: {
      target_amount: parseFloat(targetAmountStr),
      repeating_type: budgetRepeatingType,
      schedule: {
        period: schedulePeriod,
      },
    },
  };
}

function validate(request: CreateBudgetRequest): boolean {
  if (request.name.trim().length == 0) {
    return false;
  }

  return request.target === null || validateTarget(request.target);
}

function validateTarget(request: CreateBudgetTargetRequest): boolean {
  if ("OneTime" in request) {
    return request.OneTime.target_amount > 0;
  }

  if (request.Repeating.target_amount <= 0) {
    return false;
  }
  const now = new Date();
  now.setHours(0, 0, 0, 0);
  const period = request.Repeating.schedule.period;

  if ("Weekly" in period) {
    if (new Date(period.Weekly.starting_on) < now) {
      return false;
    }
  }
  if ("Fortnightly" in period) {
    if (new Date(period.Fortnightly.starting_on) < now) {
      return false;
    }
  }
  if ("Monthly" in period) {
    if (new Date(period.Monthly.starting_on) < now) {
      return false;
    }
  }
  if ("Yearly" in period) {
    if (new Date(period.Yearly.starting_on) < now) {
      return false;
    }
  }

  if ("Custom" in period) {
    return period.Custom.every_x_periods >= 1;
  }

  return true;
}
