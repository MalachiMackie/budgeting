import { MantineProvider } from "@mantine/core";
import type { Preview } from "@storybook/react";
import React from "react";
import { MockBudgetingApi } from "../src/api/budgetingApi.mock";
import { BudgetingApiContext } from "../src/App";

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
        <MantineProvider>
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
  ],
};

export default preview;
