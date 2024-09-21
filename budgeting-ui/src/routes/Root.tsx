import { IconHome, IconMoneybag } from "@tabler/icons-react";
import { QueryClient } from "@tanstack/react-query";
import { Outlet, useLoaderData } from "react-router-dom";
import { BankAccount, BudgetingApi } from "../api/budgetingApi";
import { SideNav } from "../components/SideNav/SideNav";

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
              { label: "All", link: "/accounts" },
              ...bankAccounts.map((x) => ({
                label: `${x.name}`,
                subLabel: `$${x.balance.toFixed(2)}`,
                link: `/accounts/${x.id}`,
              })),
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
