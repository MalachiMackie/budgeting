import { MantineProvider } from "@mantine/core";
import "@mantine/core/styles.css";
import "@mantine/dates/styles.css";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createContext, useContext, useEffect, useState } from "react";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import { BudgetingApi } from "./api/budgetingApi";
import { UserIdContext, useUserId } from "./hooks/useUserId";
import { AccountPage, createAccountLoader } from "./routes/AccountPage";
import { AccountsPage, createAccountsLoader } from "./routes/AccountsPage";
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

  const router = createBrowserRouter([
    {
      path: "/",
      element: <Root />,
      loader: createRootLoader(api, userId),
      errorElement: <>Something went wrong 😱</>,
      children: [
        {
          path: "/accounts/:accountId",
          element: <AccountPage />,
          loader: createAccountLoader(api, userId),
        },
        {
          path: "/accounts",
          element: <AccountsPage />,
          loader: createAccountsLoader(api, userId),
        },
      ],
    },
  ]);

  return <RouterProvider router={router} />;
}

export default App;
