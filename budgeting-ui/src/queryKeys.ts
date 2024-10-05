import { QueryKey } from "@tanstack/react-query";

export const queryKeys = {
  budgets: {
    fetch: ["budgets"],
    create: ["create-budget"],
  },
  bankAccounts: {
    fetch: ["bank-accounts"],
    fetchSingle: (bankAccountId: string) => ["bank-accounts", bankAccountId],
    create: ["create-bank-account"],
  },
  payees: {
    create: ["create-payee"],
    fetch: (userId: string) => ["payees", userId],
  },
  transactions: {
    fetch: (bankAccountId: string) => ["transactions", bankAccountId],
    create: ["create-transaction"],
  },
} satisfies ContainsQuery;

type MakesQueryKey = QueryKey | ((...params: any[]) => QueryKey);

type ContainsQuery = {
  [key: string]: MakesQueryKey | ContainsQuery;
};
