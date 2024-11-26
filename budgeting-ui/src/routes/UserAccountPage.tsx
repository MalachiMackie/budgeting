import { Button } from "@mantine/core";
import { IconPencil } from "@tabler/icons-react";
import {
  QueryClient,
  queryOptions,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { useState } from "react";
import { Client, Schedule } from "../api/client";
import { useBudgetingApi } from "../App";
import { useUserId } from "../hooks/useUserId";
import { queryKeys } from "../queryKeys";
import { EditUser } from "../views/EditUser";

export function UserAccountPage(): JSX.Element {
  const userId = useUserId();
  const api = useBudgetingApi();

  const [showEditUser, setShowEditUser] = useState(false);

  const {
    data: { data: user },
  } = useSuspenseQuery(createQueryOptions(api, userId));

  return (
    <div style={{ display: "flex", flexDirection: "column" }}>
      <div>
        <Button onClick={() => setShowEditUser(true)}>
          <IconPencil />
        </Button>
      </div>
      <span>name: {user.name}</span>
      <span>email: {user.email}</span>
      <span>
        pay frequency:{" "}
        {!user.pay_frequency ? "None" : scheduleToString(user.pay_frequency)}
      </span>
      {showEditUser && (
        <EditUser
          onCancel={() => setShowEditUser(false)}
          onSuccess={() => setShowEditUser(false)}
          user={user}
        />
      )}
    </div>
  );
}

function scheduleToString(schedule: Schedule): string {
  switch (schedule.period.type) {
    case "Weekly":
      return `Weekly starting on ${schedule.period.starting_on}`;
    case "Fortnightly":
      return `Fortnightly starting on ${schedule.period.starting_on}`;
    case "Monthly":
      return `Monthly starting on ${schedule.period.starting_on}`;
    case "Yearly":
      return `Yearly starting on ${schedule.period.starting_on}`;
    case "Custom":
      return `Every ${schedule.period.every_x_periods} ${schedule.period.period}`;
  }
}

function createQueryOptions(api: Client, userId: string) {
  return queryOptions({
    queryKey: queryKeys.users.fetchSingle(userId),
    queryFn: () => api.getUser({ userId }),
  });
}

export function createUserAccountLoader(
  api: Client,
  queryClient: QueryClient,
  userId: string
) {
  return async () => {
    await queryClient.ensureQueryData(createQueryOptions(api, userId));
    return null;
  };
}
