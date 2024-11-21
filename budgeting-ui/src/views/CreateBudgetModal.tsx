import { Button, Flex, Modal } from "@mantine/core";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import {
  CreateBudgetRequest,
  CreateBudgetTargetRequest,
  SchedulePeriod,
} from "../api/client";
import { useBudgetingApi } from "../App";
import { useUserId } from "../hooks/useUserId";
import { queryKeys } from "../queryKeys";
import { formatDate } from "../utils/formatDate";
import {
  BudgetForm,
  BudgetFormValue,
  defaultBudgetFormValue,
} from "./BudgetForm";

type CreateBudgetModalProps = {
  onCancel: () => void;
  onSuccess: (newBudgetId: string) => void;
};

export function CreateBudgetModal({
  onCancel,
  onSuccess,
}: CreateBudgetModalProps): JSX.Element {
  const [formValue, setFormValue] = useState<BudgetFormValue>(
    defaultBudgetFormValue()
  );

  const api = useBudgetingApi();
  const userId = useUserId();
  const queryClient = useQueryClient();

  const request: CreateBudgetRequest = {
    name: formValue.name,
    target: buildCreateTargetRequest(formValue),
    user_id: userId,
  };

  // todo: error messages
  const isValid = validate(request);

  const createBudget = useMutation({
    mutationKey: queryKeys.budgets.create,
    mutationFn: () => api.createBudget({}, request),
    onSuccess: async (budgetId) => {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.budgets.fetch,
      });
      onSuccess(budgetId.data);
    },
  });

  return (
    <Modal opened onClose={onCancel} title="Create New Budget">
      <BudgetForm value={formValue} onChange={setFormValue} />
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

function buildCreateTargetRequest({
  budgetRepeatingType,
  customSchedulePeriodTimes,
  customSchedulePeriodType,
  hasTarget,
  schedulePeriodType,
  scheduleStartingOn,
  targetAmountStr,
  targetType,
}: BudgetFormValue): CreateBudgetTargetRequest | undefined {
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

function validate(request: CreateBudgetRequest): boolean {
  if (request.name.trim().length == 0) {
    return false;
  }

  return request.target === undefined || validateTarget(request.target);
}

function validateTarget(request: CreateBudgetTargetRequest): boolean {
  if (request.type === "OneTime") {
    return request.target_amount > 0;
  }

  if (request.target_amount <= 0) {
    return false;
  }
  const now = new Date();
  now.setHours(0, 0, 0, 0);
  const period = request.schedule.period;

  if (period.type !== "Custom") {
    if (new Date(period.starting_on) < now) {
      return false;
    }
  }

  if (period.type === "Custom") {
    return period.every_x_periods >= 1;
  }

  return true;
}
