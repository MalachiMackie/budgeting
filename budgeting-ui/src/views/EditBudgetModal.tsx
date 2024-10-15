import { Button, Flex, Modal } from "@mantine/core";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import {
  Budget,
  SchedulePeriod,
  UpdateBudgetRequest,
  UpdateBudgetTargetRequest,
} from "../api/budgetingApi";
import { useBudgetingApi } from "../App";
import { queryKeys } from "../queryKeys";
import { formatDate } from "../utils/formatDate";
import {
  BudgetForm,
  BudgetFormValue,
  defaultBudgetFormValue,
} from "./BudgetForm";

export type EditBudgetModalProps = {
  budget: Budget;
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

function createFormValue(budget: Budget): BudgetFormValue {
  const formValue: BudgetFormValue = {
    ...defaultBudgetFormValue(),
    name: budget.name,
    hasTarget: budget.target !== null,
  };

  if (budget.target === null) {
    return formValue;
  }

  formValue.targetType = "OneTime" in budget.target ? "OneTime" : "Repeating";

  if ("OneTime" in budget.target) {
    formValue.targetAmountStr = budget.target.OneTime.target_amount.toFixed(2);
    return formValue;
  }
  const schedule = budget.target.Repeating.schedule;

  formValue.targetAmountStr = budget.target.Repeating.target_amount.toFixed(2);
  formValue.budgetRepeatingType = budget.target.Repeating.repeating_type;

  if ("Weekly" in schedule.period) {
    formValue.schedulePeriodType = "Weekly";
    formValue.scheduleStartingOn = new Date(schedule.period.Weekly.starting_on);
  } else if ("Fortnightly" in schedule.period) {
    formValue.schedulePeriodType = "Fortnightly";
    formValue.scheduleStartingOn = new Date(
      schedule.period.Fortnightly.starting_on
    );
  } else if ("Monthly" in schedule.period) {
    formValue.schedulePeriodType = "Monthly";
    formValue.scheduleStartingOn = new Date(
      schedule.period.Monthly.starting_on
    );
  } else if ("Yearly" in schedule.period) {
    formValue.schedulePeriodType = "Yearly";
    formValue.scheduleStartingOn = new Date(schedule.period.Yearly.starting_on);
  } else {
    formValue.schedulePeriodType = "Custom";
    formValue.customSchedulePeriodType = schedule.period.Custom.period;
    formValue.customSchedulePeriodTimes =
      schedule.period.Custom.every_x_periods;
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
}: BudgetFormValue): UpdateBudgetTargetRequest | null {
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
