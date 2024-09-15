import { IconHome, IconMoneybag } from "@tabler/icons-react";
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
            links: bankAccounts.map((x) => ({
              label: `${x.name}`,
              endLabel: "$0.00",
              link: `/accounts/${x.id}`,
            })),
          },
        ]}
      />
      <div style={{ flexGrow: 1 }}>
        <Outlet />
      </div>
    </div>
  );
}

export function createRootLoader(api: BudgetingApi, userId: string) {
  return () => {
    return api.getBankAccounts(userId);
  };
}
