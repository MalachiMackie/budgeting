import {
  IconHome,
  IconMan,
  IconMoneybag,
  IconPlus,
  IconUsersGroup,
} from "@tabler/icons-react";
import {
  QueryClient,
  queryOptions,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { useState } from "react";
import { Outlet } from "react-router-dom";
import { Client } from "../api/client";
import { useBudgetingApi } from "../App";
import { SideNav, SideNavProps } from "../components/SideNav/SideNav";
import { useUserId } from "../hooks/useUserId";
import { queryKeys } from "../queryKeys";
import { formatCurrency } from "../utils/formatCurrency";
import { CreateBankAccountModal } from "../views/CreateBankAccountModal";

export function Root(): JSX.Element {
  const api = useBudgetingApi();
  const userId = useUserId();
  const { data: bankAccounts } = useSuspenseQuery(
    createQueryOptions(api, userId)
  );

  const [showCreateBankAccount, setShowCreateBankAccount] = useState(false);

  return (
    <div style={{ display: "flex" }}>
      <SideNav
        items={[
          {
            type: "link",
            id: "home",
            label: "Home",
            link: "/",
            icon: IconHome,
          },
          {
            type: "link",
            id: "budgets",
            label: "Budgets",
            link: "/budgets",
            icon: IconHome,
          },
          {
            type: "group",
            label: "Accounts",
            icon: IconMoneybag,
            id: "accounts",
            links: [
              {
                type: "link",
                label: "All",
                id: "all",
                link: "/accounts",
                icon: IconUsersGroup,
              },
              ...bankAccounts.data.map(
                (x) =>
                  ({
                    type: "link",
                    id: x.id,
                    label: `${x.name}`,
                    subLabel: formatCurrency(x.balance),
                    link: `/accounts/${x.id}`,
                  }) satisfies Extract<
                    SideNavProps["items"][number],
                    { type: "group" }
                  >["links"][number]
              ),
              {
                label: "Create New",
                type: "button",
                id: "create-new",
                onClick: () => {
                  setShowCreateBankAccount(true);
                },
                icon: IconPlus,
              },
            ],
          },
          {
            type: "link",
            id: "payees",
            label: "Payees",
            link: "/payees",
            icon: IconMan,
          },
          {
            type: "link",
            id: "user-account",
            label: "User Account",
            link: "/user-account",
          },
        ]}
      />
      <div style={{ flexGrow: 1, padding: "1rem" }}>
        <Outlet />
        {showCreateBankAccount && (
          <CreateBankAccountModal
            onClose={() => setShowCreateBankAccount(false)}
            onSuccess={() => {
              setShowCreateBankAccount(false);
            }}
          />
        )}
      </div>
    </div>
  );
}

function createQueryOptions(api: Client, userId: string) {
  return queryOptions({
    queryKey: queryKeys.bankAccounts.fetch,
    queryFn: () => api.getBankAccounts(userId),
  });
}

export function createRootLoader(
  api: Client,
  queryClient: QueryClient,
  userId: string
) {
  return async () => {
    await queryClient.ensureQueryData(createQueryOptions(api, userId));
    return null;
  };
}
