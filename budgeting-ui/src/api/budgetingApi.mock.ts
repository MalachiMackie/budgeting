import {
  BudgetingApi,
  CreateTransactionRequest,
  Payee,
  Transaction,
} from "./budgetingApi";

export const MockBudgetingApi: BudgetingApi & {
  transactions: Transaction[];
  payees: Payee[];
} = {
  transactions: [
    {
      id: "my-id",
      payee_id: "payee-id",
      amount_dollars: 15,
      amount_cents: 32,
      date: "2024-09-08",
    },
  ],
  payees: [
    {
      id: "payee-id",
      name: "My Shop",
    },
  ],
  getTransactions: async function (): Promise<Transaction[]> {
    console.log("getting transactions");
    return this.transactions;
  },
  getPayees: async function (): Promise<Payee[]> {
    return this.payees;
  },
  createTransaction: function (_: CreateTransactionRequest): Promise<void> {
    return Promise.resolve();
  },
};
