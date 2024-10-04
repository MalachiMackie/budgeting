import {
  Button,
  Checkbox,
  Flex,
  Modal,
  NumberInput,
  SegmentedControl,
  TextInput,
} from "@mantine/core";
import { DatePickerInput } from "@mantine/dates";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import {
  BudgetRepeatingType,
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

  const createBudget = useMutation({
    mutationKey: queryKeys.budgets.create,
    mutationFn: () =>
      api.createBudget({
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
      }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.budgets.fetch,
      });
      onSuccess();
    },
  });

  return (
    <Modal opened onClose={onCancel} title="Create New Budget">
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
      {schedulePeriodType === "Custom" && (
        <>
          Every
          <NumberInput
            value={customSchedulePeriodTimes}
            onChange={(x) =>
              typeof x === "number" && setCustomSchedulePeriodTimes(x)
            }
          />
          <SegmentedControl
            data={["Weekly", "Fortnightly", "Monthly", "Yearly"]}
            value={customSchedulePeriodType}
            onChange={(x) =>
              setCustomSchedulePeriodType(x as typeof customSchedulePeriodType)
            }
          />
        </>
      )}
      <Flex gap={"0.5rem"} mt={"1rem"} justify={"flex-end"}>
        <Button variant="subtle" onClick={onCancel}>
          Cancel
        </Button>
        <Button onClick={() => createBudget.mutate()}>Create</Button>
      </Flex>
    </Modal>
  );
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
