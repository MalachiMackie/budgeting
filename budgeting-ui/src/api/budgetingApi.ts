export const BudgetingApi = {
  async getTransactions(bankAccountId: string): Promise<Transaction[]> {
    let result = await fetch(
      `http://localhost:3000/api/bank-accounts/${bankAccountId}/transactions`
    );
    let json = await result.json();
    return json as Transaction[];
  },
  async createTransaction(
    request: CreateTransactionRequest,
    bankAccountId: string
  ): Promise<void> {
    await fetch(
      `http://localhost:3000/api/bank-accounts/${bankAccountId}/transactions`,
      {
        method: "POST",
        body: JSON.stringify(request),
        headers: { "Content-Type": "application/json" },
      }
    );
  },
  async getPayees(userId: string): Promise<Payee[]> {
    let result = await fetch(
      `http://localhost:3000/api/payees?user_id=${userId}`
    );
    let json = await result.json();
    return json as Payee[];
  },
  async getUsers(): Promise<User[]> {
    let result = await fetch("http://localhost:3000/api/users");
    let json = await result.json();
    return json as User[];
  },
  async getBankAccounts(userId: string): Promise<BankAccount[]> {
    let result = await fetch(
      `http://localhost:3000/api/bank-accounts?user_id=${userId}`
    );
    let json = await result.json();
    return json as BankAccount[];
  },
};

export type BudgetingApi = typeof BudgetingApi;

export type Transaction = {
  id: string;
  payee_id: string;
  amount: number;
  date: string;
};

export type Payee = {
  id: string;
  name: string;
};

export type CreateTransactionRequest = {
  payee_id: string;
  amount: number;
  date: string;
};

export type User = {
  id: string;
  email: string;
};

export type BankAccount = {
  name: string;
  id: string;
  initial_amount: number;
};
