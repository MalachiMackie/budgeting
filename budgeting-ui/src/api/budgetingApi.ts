export const BudgetingApi = {
  async getTransactions(): Promise<Transaction[]> {
    let result = await fetch("http://localhost:3000/api/transactions");
    let json = await result.json();
    return json as Transaction[];
  },
  async createTransaction(request: CreateTransactionRequest): Promise<void> {
    await fetch("http://localhost:3000/api/transactions", {
      method: "POST",
      body: JSON.stringify(request),
      headers: { "Content-Type": "application/json" },
    });
  },
  async getPayees(): Promise<Payee[]> {
    let result = await fetch("http://localhost:3000/api/payees");
    let json = await result.json();
    return json as Payee[];
  },
};

export type BudgetingApi = typeof BudgetingApi;

export type Transaction = {
  id: string;
  payee_id: string;
  amount_dollars: number;
  amount_cents: number;
  date: string;
};

export type Payee = {
  id: string;
  name: string;
};

export type CreateTransactionRequest = {
  payee_id: string;
  amount_dollars: number;
  amount_cents: number;
  date: string;
};
