import {
  BudgetingApi,
  CreateTransactionRequest,
  Payee,
  Transaction,
} from "./budgetingApi";

export const MockBudgetingApi: BudgetingApi = {
  getTransactions: async function (): Promise<Transaction[]> {
    return [
      {
        id: "my-id",
        payee_id: "payee-id",
        amount_dollars: 15,
        amount_cents: 32,
        date: "2024-09-08",
      },
    ];
  },
  getPayees: async function (): Promise<Payee[]> {
    return [
      {
        id: "payee-id",
        name: "My Shop",
      },
    ];
  },
  createTransaction: function (_: CreateTransactionRequest): Promise<void> {
    return Promise.resolve();
  },
};
