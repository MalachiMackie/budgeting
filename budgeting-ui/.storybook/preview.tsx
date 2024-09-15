import { MantineProvider } from "@mantine/core";
import type { Preview } from "@storybook/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import React from "react";
import { MockBudgetingApi } from "../src/api/budgetingApi.mock";
import { BudgetingApiContext } from "../src/App";

const queryClient = new QueryClient();

const preview: Preview = {
  parameters: {
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
  },
  decorators: [
    (Story, _) => {
      return (
        <MantineProvider defaultColorScheme="dark">
          <Story />
        </MantineProvider>
      );
    },
    (Story) => {
      return (
        <BudgetingApiContext.Provider value={MockBudgetingApi}>
          <Story />
        </BudgetingApiContext.Provider>
      );
    },
    (Story) => {
      return (
        <QueryClientProvider client={queryClient}>
          <Story />
        </QueryClientProvider>
      );
    },
  ],
};

export default preview;
