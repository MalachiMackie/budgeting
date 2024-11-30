import { Button, Flex, Modal } from "@mantine/core";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import {
  GetBudgetResponse,
  SchedulePeriod,
  UpdateBudgetRequest,
  UpdateBudgetTargetRequest,
} from "../api/client";
import { useBudgetingApi } from "../App";
import { queryKeys } from "../queryKeys";
import { formatDate } from "../utils/formatDate";
import {
  BudgetForm,
  BudgetFormValue,
  defaultBudgetFormValue,
} from "./BudgetForm";

export type EditBudgetModalProps = {
  budget: GetBudgetResponse;
  onCancel: () => void;
  onSuccess: () => void;
};

export function EditBudgetModal({
  budget,
  onCancel,
  onSuccess,
}: EditBudgetModalProps): JSX.Element {
  const [formValue, setFormValue] = useState<BudgetFormValue>(
    createFormValue(budget)
  );

  const api = useBudgetingApi();
  const queryClient = useQueryClient();

  const request: UpdateBudgetRequest = {
    name: formValue.name,
    target: buildUpdateTargetRequest(formValue),
  };

  const updateBudget = useMutation({
    mutationKey: queryKeys.budgets.edit(budget.id),
    mutationFn: () => api.updateBudget(budget.id, request),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.budgets.fetch,
      });
      onSuccess();
    },
  });

  return (
    <Modal opened onClose={onCancel} title="Create New Budget">
      <BudgetForm value={formValue} onChange={setFormValue} />
      <Flex gap={"0.5rem"} mt={"1rem"} justify={"flex-end"}>
        <Button variant="subtle" onClick={onCancel}>
          Cancel
        </Button>
        <Button onClick={() => updateBudget.mutate()}>Save</Button>
      </Flex>
    </Modal>
  );
}

function createFormValue(budget: GetBudgetResponse): BudgetFormValue {
  const formValue: BudgetFormValue = {
    ...defaultBudgetFormValue(),
    name: budget.name,
    hasTarget: budget.target !== null,
  };

  if (!budget.target) {
    return formValue;
  }

  formValue.targetType = budget.target?.type;

  if (budget.target.type === "OneTime") {
    formValue.targetAmountStr = budget.target.target_amount.toFixed(2);
    return formValue;
  }
  const schedule = budget.target.schedule;

  formValue.targetAmountStr = budget.target.target_amount.toFixed(2);
  formValue.budgetRepeatingType = budget.target.repeating_type;

  if (schedule.period.type === "Weekly") {
    formValue.schedulePeriodType = "Weekly";
    formValue.scheduleStartingOn = new Date(schedule.period.starting_on);
  } else if (schedule.period.type === "Fortnightly") {
    formValue.schedulePeriodType = "Fortnightly";
    formValue.scheduleStartingOn = new Date(schedule.period.starting_on);
  } else if (schedule.period.type === "Monthly") {
    formValue.schedulePeriodType = "Monthly";
    formValue.scheduleStartingOn = new Date(schedule.period.starting_on);
  } else if (schedule.period.type === "Yearly") {
    formValue.schedulePeriodType = "Yearly";
    formValue.scheduleStartingOn = new Date(schedule.period.starting_on);
  } else {
    formValue.schedulePeriodType = "Custom";
    formValue.customSchedulePeriodType = schedule.period.period;
    formValue.customSchedulePeriodTimes = schedule.period.every_x_periods;
  }

  return formValue;
}

function buildUpdateTargetRequest({
  budgetRepeatingType,
  customSchedulePeriodTimes,
  customSchedulePeriodType,
  hasTarget,
  schedulePeriodType,
  scheduleStartingOn,
  targetAmountStr,
  targetType,
}: BudgetFormValue): UpdateBudgetTargetRequest | undefined {
  if (!hasTarget) {
    return undefined;
  }

  if (targetType === "OneTime") {
    return {
      type: "OneTime",
      target_amount: parseFloat(targetAmountStr),
    };
  }

  const startingOnStr = formatDate(scheduleStartingOn);

  let schedulePeriod: SchedulePeriod;

  switch (schedulePeriodType) {
    case "Weekly":
      schedulePeriod = { type: "Weekly", starting_on: startingOnStr };
      break;
    case "Fortnightly":
      schedulePeriod = { type: "Fortnightly", starting_on: startingOnStr };
      break;
    case "Monthly":
      schedulePeriod = { type: "Monthly", starting_on: startingOnStr };
      break;
    case "Yearly":
      schedulePeriod = { type: "Yearly", starting_on: startingOnStr };
      break;
    case "Custom":
      schedulePeriod = {
        type: "Custom",
        every_x_periods: customSchedulePeriodTimes,
        period: customSchedulePeriodType,
      };
      break;
  }

  return {
    type: "Repeating",
    target_amount: parseFloat(targetAmountStr),
    repeating_type: budgetRepeatingType,
    schedule: {
      period: schedulePeriod,
    },
  };
}
