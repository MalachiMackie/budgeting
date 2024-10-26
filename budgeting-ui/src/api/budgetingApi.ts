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
  ): Promise<string> {
    let result = await fetch(
      `http://localhost:3000/api/bank-accounts/${bankAccountId}/transactions`,
      {
        method: "POST",
        body: JSON.stringify(request),
        headers: { "Content-Type": "application/json" },
      }
    );
    let json = await result.json();

    return json as string;
  },
  async updateTransaction(
    transactionId: string,
    request: UpdateTransactionRequest
  ): Promise<void> {
    await fetch(`http://localhost:3000/api/transactions/${transactionId}`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(request),
    });
  },
  async getPayees(userId: string): Promise<Payee[]> {
    let result = await fetch(
      `http://localhost:3000/api/payees?user_id=${userId}`
    );
    let json = await result.json();
    return json as Payee[];
  },
  async createPayee(request: CreatePayeeRequest): Promise<string> {
    let result = await fetch(`http://localhost:3000/api/payees`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(request),
    });

    let json = await result.json();
    return json as string;
  },
  async getUsers(): Promise<User[]> {
    let result = await fetch("http://localhost:3000/api/users");
    let json = await result.json();
    return json as User[];
  },
  async getBankAccount(
    accountId: string,
    userId: string
  ): Promise<BankAccount> {
    const result = await fetch(
      `http://localhost:3000/api/bank-accounts/${accountId}?user_id=${userId}`
    );
    const json = await result.json();
    return json as BankAccount;
  },
  async getBankAccounts(userId: string): Promise<BankAccount[]> {
    let result = await fetch(
      `http://localhost:3000/api/bank-accounts?user_id=${userId}`
    );
    let json = await result.json();
    return json as BankAccount[];
  },
  async getBudgets(userId: string): Promise<Budget[]> {
    let result = await fetch(
      `http://localhost:3000/api/budgets?user_id=${userId}`
    );

    let json = await result.json();
    return json as Budget[];
  },
  async createBankAccount(request: CreateBankAccountRequest): Promise<string> {
    let result = await fetch(`http://localhost:3000/api/bank-accounts`, {
      method: "POST",
      body: JSON.stringify(request),
      headers: { "Content-Type": "application/json" },
    });

    let json = await result.json();
    return json as string;
  },
  async updateBankAccount(
    bankAccountId: string,
    userId: string,
    request: UpdateBankAccountRequest
  ): Promise<void> {
    await fetch(
      `http://localhost:3000/api/bank-accounts/${bankAccountId}?user_id=${userId}`,
      {
        method: "PUT",
        body: JSON.stringify(request),
        headers: JsonContentTypeHeaders,
      }
    );
  },
  async deleteBankAccount(
    bankAccountId: string,
    userId: string
  ): Promise<void> {
    await fetch(
      `http://localhost:3000/api/bank-accounts/${bankAccountId}?user_id=${userId}`,
      {
        method: "DELETE",
      }
    );
  },
  async createBudget(request: CreateBudgetRequest): Promise<string> {
    let result = await fetch(`http://localhost:3000/api/budgets`, {
      method: "POST",
      body: JSON.stringify(request),
      headers: { "Content-Type": "application/json" },
    });

    let json = await result.json();

    return json as string;
  },
  async updateBudget(id: string, request: UpdateBudgetRequest): Promise<void> {
    await fetch(`http://localhost:3000/api/budgets/${id}`, {
      method: "PUT",
      body: JSON.stringify(request),
      headers: JsonContentTypeHeaders,
    });
  },
};

export type BudgetingApi = typeof BudgetingApi;

export type Transaction = {
  id: string;
  payee_id: string;
  amount: number;
  date: string;
  budget_id: string;
};

export type Payee = {
  id: string;
  name: string;
};

export type CreateTransactionRequest = {
  payee_id: string;
  amount: number;
  date: string;
  budget_id: string;
};

export type UpdateTransactionRequest = {
  payee_id: string;
  amount: number;
  date: string;
  budget_id: string;
};

export type User = {
  id: string;
  email: string;
};

export type BankAccount = {
  name: string;
  id: string;
  initial_amount: number;
  balance: number;
};

export type CreateBankAccountRequest = {
  name: string;
  initial_amount: number;
  user_id: string;
};

export type Budget = {
  id: string;
  name: string;
  target: BudgetTarget | null;
  user_id: string;
};

export type CreateBudgetRequest = {
  name: string;
  target: CreateBudgetTargetRequest | null;
  user_id: string;
};

export type UpdateBudgetRequest = {
  name: string;
  target: UpdateBudgetTargetRequest | null;
};

export type UpdateBudgetTargetRequest =
  | { OneTime: { target_amount: number } }
  | {
      Repeating: {
        target_amount: number;
        repeating_type: BudgetRepeatingType;
        schedule: UpdateScheduleRequest;
      };
    };

export type UpdateScheduleRequest = CreateScheduleRequest;

export type BudgetTarget =
  | { OneTime: { target_amount: number } }
  | {
      Repeating: {
        target_amount: number;
        repeating_type: BudgetRepeatingType;
        schedule: Schedule;
      };
    };

// TODO: ideally this would map out BudgetTarget but replace Schedule with CreateScheduleRequest
export type CreateBudgetTargetRequest =
  | { OneTime: { target_amount: number } }
  | {
      Repeating: {
        target_amount: number;
        repeating_type: BudgetRepeatingType;
        schedule: CreateScheduleRequest;
      };
    };

export type BudgetRepeatingType = "BuildUpTo" | "RequireRepeating";

export type Schedule = { id: string; period: SchedulePeriod };

export type CreateScheduleRequest = Omit<Schedule, "id">;

type NormalSchedulePeriod = { starting_on: string };

export type SchedulePeriod =
  | { Weekly: NormalSchedulePeriod }
  | { Fortnightly: NormalSchedulePeriod }
  | { Monthly: NormalSchedulePeriod }
  | { Yearly: NormalSchedulePeriod }
  | { Custom: { period: SchedulePeriodType; every_x_periods: number } };

export type SchedulePeriodType =
  | "Weekly"
  | "Fortnightly"
  | "Monthly"
  | "Yearly";

export type CreatePayeeRequest = {
  name: string;
  user_id: string;
};

export type UpdateBankAccountRequest = {
  name: string;
};

const JsonContentTypeHeaders = {
  "Content-Type": "application/json",
};
