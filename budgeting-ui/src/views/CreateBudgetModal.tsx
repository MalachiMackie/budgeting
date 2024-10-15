import { Button, Flex, Modal } from "@mantine/core";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import {
  CreateBudgetRequest,
  CreateBudgetTargetRequest,
  SchedulePeriod,
} from "../api/budgetingApi";
import { useBudgetingApi } from "../App";
import { useUserId } from "../hooks/useUserId";
import { queryKeys } from "../queryKeys";
import { formatDate } from "../utils/formatDate";
import { BudgetForm, BudgetFormValue, defaultBudgetFormValue } from "./BudgetForm";

type CreateBudgetModalProps = {
  onCancel: () => void;
  onSuccess: (newBudgetId: string) => void;
};

export function CreateBudgetModal({
  onCancel,
  onSuccess,
}: CreateBudgetModalProps): JSX.Element {
  const [formValue, setFormValue] = useState<BudgetFormValue>(defaultBudgetFormValue());

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
    mutationFn: () => api.createBudget(request),
    onSuccess: async (budgetId) => {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.budgets.fetch,
      });
      onSuccess(budgetId);
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
}: BudgetFormValue): CreateBudgetTargetRequest | null {
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
