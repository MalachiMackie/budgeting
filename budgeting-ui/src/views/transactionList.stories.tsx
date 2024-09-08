import { Meta, StoryObj } from "@storybook/react";
import TransactionList from "./transactionList";

export default {
  component: TransactionList,
  args: {},
} satisfies Meta<typeof TransactionList>;

export const Default: StoryObj<typeof TransactionList> = {};
