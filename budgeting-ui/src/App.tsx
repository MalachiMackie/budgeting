import { MantineProvider } from "@mantine/core";
import "@mantine/core/styles.css";
import "@mantine/dates/styles.css";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createContext, useContext, useEffect, useState } from "react";
import { BudgetingApi } from "./api/budgetingApi";
import { AppBar } from "./views/AppBar/AppBar";
import { BankAccountList } from "./views/BankAccountList";

export const BudgetingApiContext = createContext<BudgetingApi>(null!);
export const UserContext = createContext<{ userId: string | null }>({
  userId: null,
});

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
          {user && (
            <UserContext.Provider value={{userId: user}}>
              <AppBar />
              <BankAccountList userId={user} />
            </UserContext.Provider>
          )}
        </QueryClientProvider>
      </BudgetingApiContext.Provider>
    </MantineProvider>
  );
}

export default App;
