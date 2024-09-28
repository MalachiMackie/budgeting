import {
  IconHome,
  IconMoneybag,
  IconPlus,
  IconUsersGroup,
} from "@tabler/icons-react";
import { QueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Outlet, useLoaderData, useRevalidator } from "react-router-dom";
import { BankAccount, BudgetingApi } from "../api/budgetingApi";
import { SideNav, SideNavProps } from "../components/SideNav/SideNav";
import { CreateBankAccountModal } from "../views/CreateBankAccountModal";

export function Root(): JSX.Element {
  const bankAccounts = useLoaderData() as BankAccount[];
  const [showCreateBankAccount, setShowCreateBankAccount] = useState(false);

  const { revalidate } = useRevalidator();

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
              ...bankAccounts.map(
                (x) =>
                  ({
                    type: "link",
                    id: x.id,
                    label: `${x.name}`,
                    subLabel: `$${x.balance.toFixed(2)}`,
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
        ]}
      />
      <div style={{ flexGrow: 1, padding: "1rem" }}>
        <Outlet />
        {showCreateBankAccount && (
          <CreateBankAccountModal
            onClose={() => setShowCreateBankAccount(false)}
            onSuccess={() => {
              setShowCreateBankAccount(false);
              revalidate();
            }}
          />
        )}
      </div>
    </div>
  );
}

export function createRootLoader(
  api: BudgetingApi,
  queryClient: QueryClient,
  userId: string
) {
  return () => {
    return queryClient.fetchQuery({
      queryKey: ["bank-accounts"],
      queryFn: () => api.getBankAccounts(userId),
    });
  };
}
