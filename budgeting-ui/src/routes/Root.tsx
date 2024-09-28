import {
  IconHome,
  IconMoneybag,
  IconPlus,
  IconUsersGroup,
} from "@tabler/icons-react";
import { QueryClient } from "@tanstack/react-query";
import { Outlet, useLoaderData } from "react-router-dom";
import { BankAccount, BudgetingApi } from "../api/budgetingApi";
import { SideNav, SideNavProps } from "../components/SideNav/SideNav";

export function Root(): JSX.Element {
  const bankAccounts = useLoaderData() as BankAccount[];

  return (
    <div style={{ display: "flex" }}>
      <SideNav
        items={[
          { type: "link", label: "Home", link: "/", icon: IconHome },
          {
            type: "group",
            label: "Accounts",
            icon: IconMoneybag,
            links: [
              {
                type: "link",
                label: "All",
                link: "/accounts",
                icon: IconUsersGroup,
              },
              ...bankAccounts.map(
                (x) =>
                  ({
                    type: "link",
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
                onClick: () => {
                  alert("hi");
                },
                icon: IconPlus,
              },
            ],
          },
        ]}
      />
      <div style={{ flexGrow: 1, padding: "1rem" }}>
        <Outlet />
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
