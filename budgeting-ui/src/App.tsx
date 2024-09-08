import { MantineProvider } from "@mantine/core";
import "@mantine/core/styles.css";
import "@mantine/dates/styles.css";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createContext, useContext } from "react";
import { BudgetingApi } from "./api/budgetingApi";
import TransactionList from "./views/transactionList";

export const BudgetingApiContext = createContext(BudgetingApi);

export function useBudgetingApi(): BudgetingApi {
  return useContext(BudgetingApiContext);
}

const queryClient = new QueryClient();

function App() {
  return (
    <MantineProvider>
      <BudgetingApiContext.Provider value={BudgetingApi}>
        <QueryClientProvider client={queryClient}>
          <TransactionList />
        </QueryClientProvider>
      </BudgetingApiContext.Provider>
    </MantineProvider>
  );
}

export default App;
