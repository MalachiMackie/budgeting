import {
  BankAccount,
  BudgetingApi,
  Payee,
  Transaction,
  User,
} from "./budgetingApi";

export const MockBudgetingApi: BudgetingApi = {
  getTransactions: async function (_): Promise<Transaction[]> {
    return [
      {
        id: "my-id",

        payee_id: "payee-id",
        amount: 15.32,
        date: "2024-09-08",
      },
    ];
  },
  getPayees: async function (_): Promise<Payee[]> {
    return [
      {
        id: "payee-id",
        name: "My Shop",
      },
    ];
  },
  createTransaction: function (_, __): Promise<void> {
    return Promise.resolve();
  },
  getUsers: function (): Promise<User[]> {
    return Promise.resolve([{ id: "user-id", email: "my@email.com" }]);
  },
  getBankAccounts: function (_: string): Promise<BankAccount[]> {
    return Promise.resolve([
      {
        name: "bank-account",
        id: "bank-account-id",
        initial_amount: 15.12,
      },
    ]);
  },
};
