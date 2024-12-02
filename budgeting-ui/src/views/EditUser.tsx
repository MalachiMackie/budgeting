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
  SchedulePeriod,
  SchedulePeriodType,
  UpdateScheduleRequest,
  User,
} from "../api/client";
import { useBudgetingApi } from "../App";
import { queryKeys } from "../queryKeys";
import { formatDateForApi } from "../utils/formatDate";

export type EditUserProps = {
  user: User;
  onSuccess: () => void;
  onCancel: () => void;
};

export function EditUser({
  user,
  onSuccess,
  onCancel,
}: EditUserProps): JSX.Element {
  const [value, setValue] = useState(buildFormValue(user));

  const api = useBudgetingApi();
  const queryClient = useQueryClient();

  const saveUser = useMutation({
    mutationKey: queryKeys.users.edit(user.id),
    mutationFn: () =>
      api.updateUser(
        { userId: user.id },
        {
          name: value.name,
          pay_frequency: buildUpdateScheduleRequest(value),
        }
      ),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.users.fetchSingle(user.id),
      });
      onSuccess();
    },
  });

  return (
    <Modal opened onClose={onCancel} title="Edit User">
      <Flex gap="0.5rem" direction={"column"}>
        <TextInput
          label="Name"
          value={value.name}
          onChange={(e) => setValue({ ...value, name: e.currentTarget.value })}
        />
        <Checkbox
          label="Pay Frequency"
          checked={value.hasPayFrequency}
          onChange={(e) =>
            setValue({ ...value, hasPayFrequency: e.currentTarget.checked })
          }
        />
        {value.hasPayFrequency && (
          <>
            <SegmentedControl
              data={["Weekly", "Fortnightly", "Monthly", "Yearly", "Custom"]}
              value={value.schedulePeriodType}
              onChange={(x) =>
                setValue({
                  ...value,
                  schedulePeriodType: x as typeof value.schedulePeriodType,
                })
              }
            />
            {value.schedulePeriodType !== "Custom" && (
              <>
                <DatePickerInput
                  value={value.scheduleStartingOn}
                  label="Starting on"
                  onChange={(x) =>
                    x && setValue({ ...value, scheduleStartingOn: x })
                  }
                />
              </>
            )}
            {value.schedulePeriodType === "Custom" && (
              <>
                <Flex gap="0.5rem">
                  <span style={{ verticalAlign: "middle" }}>Every</span>
                  <NumberInput
                    value={value.customSchedulePeriodTimes}
                    onChange={(x) =>
                      typeof x === "number" &&
                      setValue({ ...value, customSchedulePeriodTimes: x })
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
                    setValue({
                      ...value,
                      customSchedulePeriodType:
                        x as typeof value.customSchedulePeriodType,
                    })
                  }
                />
              </>
            )}
          </>
        )}
        <Button onClick={() => saveUser.mutate()}>Save</Button>
      </Flex>
    </Modal>
  );
}

function buildUpdateScheduleRequest({
  customSchedulePeriodTimes,
  customSchedulePeriodType,
  hasPayFrequency,
  schedulePeriodType,
  scheduleStartingOn,
}: UserFormValue): UpdateScheduleRequest | undefined {
  if (!hasPayFrequency) {
    return undefined;
  }

  const startingOnStr = formatDateForApi(scheduleStartingOn);

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

  return { period: schedulePeriod };
}

function buildFormValue(user: User): UserFormValue {
  return {
    name: user.name,
    hasPayFrequency: !!user.pay_frequency,
    scheduleStartingOn:
      user.pay_frequency && user.pay_frequency.period.type !== "Custom"
        ? new Date(user.pay_frequency.period.starting_on)
        : new Date(),
    schedulePeriodType: user.pay_frequency
      ? user.pay_frequency.period.type
      : "Weekly",
    customSchedulePeriodType:
      user.pay_frequency && user.pay_frequency.period.type === "Custom"
        ? user.pay_frequency.period.period
        : "Weekly",
    customSchedulePeriodTimes:
      user.pay_frequency && user.pay_frequency.period.type === "Custom"
        ? user.pay_frequency.period.every_x_periods
        : 1,
  };
}

type UserFormValue = {
  name: string;
  hasPayFrequency: boolean;
  schedulePeriodType: SchedulePeriodType | "Custom";
  scheduleStartingOn: Date;
  customSchedulePeriodTimes: number;
  customSchedulePeriodType: SchedulePeriodType;
};

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
