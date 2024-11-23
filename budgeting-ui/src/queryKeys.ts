import { QueryKey } from "@tanstack/react-query";

export const queryKeys = {
  budgets: {
    fetch: ["budgets"],
    create: ["create-budget"],
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
