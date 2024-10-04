import { MantineProvider } from "@mantine/core";
import "@mantine/core/styles.css";
import "@mantine/dates/styles.css";
import {
  QueryClient,
  QueryClientProvider,
  useQueryClient,
} from "@tanstack/react-query";
import { createContext, useContext, useEffect, useState } from "react";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import { BudgetingApi } from "./api/budgetingApi";
import { UserIdContext, useUserId } from "./hooks/useUserId";
import { AccountPage, createAccountLoader } from "./routes/AccountPage";
import { AccountsPage, createAccountsLoader } from "./routes/AccountsPage";
import { BudgetsPage, createBudgetsLoader } from "./routes/BudgetsPage";
import { createRootLoader, Root } from "./routes/Root";

export const BudgetingApiContext = createContext<BudgetingApi>(null!);

export function useBudgetingApi(): BudgetingApi {
  return useContext(BudgetingApiContext);
}

const queryClient = new QueryClient();

function App() {
  let [user, setUser] = useState<string | null>(null);
  let budgetingApi = BudgetingApi;

  // for now, just load the first user
  useEffect(() => {
    let load = async () => {
      let users = await budgetingApi.getUsers();
      if (users.length == 0) {
        throw new Error("No users!");
      }
      setUser(users[0].id);
    };

    void load();
  }, []);

  return (
    <MantineProvider defaultColorScheme="dark">
      <BudgetingApiContext.Provider value={budgetingApi}>
        <QueryClientProvider client={queryClient}>
          {user !== null && (
            <UserIdContext.Provider value={{ userId: user }}>
              <BudgetingRouterWrapper />
            </UserIdContext.Provider>
          )}
        </QueryClientProvider>
      </BudgetingApiContext.Provider>
    </MantineProvider>
  );
}

// temporary until actually figure out user id
export function BudgetingRouterWrapper() {
  const api = useBudgetingApi();
  const userId = useUserId();
  const queryClient = useQueryClient();

  const router = createBrowserRouter([
    {
      path: "/",
      element: <Root />,
      loader: createRootLoader(api, queryClient, userId),
      errorElement: <>Something went wrong 😱</>,
      children: [
        {
          path: "/accounts/:accountId",
          element: <AccountPage />,
          loader: createAccountLoader(api, queryClient, userId),
        },
        {
          path: "/accounts",
          element: <AccountsPage />,
          loader: createAccountsLoader(api, queryClient, userId),
        },
        {
          path: "/budgets",
          element: <BudgetsPage />,
          loader: createBudgetsLoader(api, queryClient, userId),
        },
      ],
    },
  ]);

  return <RouterProvider router={router} />;
}

export default App;
