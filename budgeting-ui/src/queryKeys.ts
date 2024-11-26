import { QueryKey } from "@tanstack/react-query";

export const queryKeys = {
  users: {
    fetchSingle: (userId: string) => ["users", userId],
    edit: (userId: string) => ["edit-user", userId],
  },
  budgets: {
    fetch: ["budgets"],
    create: ["create-budget"],
    delete: ["delete-budget"],
    edit: (budgetId: string) => ["edit-budget", budgetId],
  },
  bankAccounts: {
    fetch: ["bank-accounts"],
    fetchSingle: (bankAccountId: string) => ["bank-accounts", bankAccountId],
    create: ["create-bank-account"],
    delete: (bankAccountId: string) => ["delete-bank-account", bankAccountId],
    edit: (bankAccountId: string) => ["edit-bank-account", bankAccountId],
  },
  payees: {
    create: ["create-payee"],
    delete: ["delete-payee"],
    edit: (payeeId: string) => ["edit-payee", payeeId],
    fetch: (userId: string) => ["payees", userId],
  },
  transactions: {
    fetch: (bankAccountId: string) => ["transactions", bankAccountId],
    edit: (transactionId: string) => ["edit-transaction", transactionId],
    create: ["create-transaction"],
    delete: ["delete-transaction"],
  },
} satisfies ContainsQuery;

type MakesQueryKey = QueryKey | ((...params: any[]) => QueryKey);

type ContainsQuery = {
  [key: string]: MakesQueryKey | ContainsQuery;
};
