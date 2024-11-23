import type {
  OpenAPIClient,
  Parameters,
  UnknownParamsObject,
  OperationResponse,
  AxiosRequestConfig,
} from 'openapi-client-axios';

declare namespace Components {
    namespace Schemas {
        export interface BankAccount {
            balance: number; // float
            id: string; // uuid
            initial_amount: number; // float
            name: string;
            user_id: string; // uuid
        }
        export interface Budget {
            id: string; // uuid
            name: string;
            target?: {
                target_amount: number; // float
                type: "OneTime";
            } | {
                repeating_type: RepeatingTargetType;
                schedule: Schedule;
                target_amount: number; // float
                type: "Repeating";
            };
            user_id: string; // uuid
        }
        export type BudgetTarget = {
            target_amount: number; // float
            type: "OneTime";
        } | {
            repeating_type: RepeatingTargetType;
            schedule: Schedule;
            target_amount: number; // float
            type: "Repeating";
        };
        export interface CreateBankAccountRequest {
            initial_amount: number; // float
            name: string;
            user_id: string; // uuid
        }
        export interface CreateBudgetRequest {
            name: string;
            target?: {
                target_amount: number; // float
                type: "OneTime";
            } | {
                repeating_type: RepeatingTargetType;
                schedule: CreateScheduleRequest;
                target_amount: number; // float
                type: "Repeating";
            };
            user_id: string; // uuid
        }
        export type CreateBudgetTargetRequest = {
            target_amount: number; // float
            type: "OneTime";
        } | {
            repeating_type: RepeatingTargetType;
            schedule: CreateScheduleRequest;
            target_amount: number; // float
            type: "Repeating";
        };
        export interface CreatePayeeRequest {
            name: string;
            user_id: string; // uuid
        }
        export interface CreateScheduleRequest {
            period: SchedulePeriod;
        }
        export interface CreateTransactionRequest {
            amount: number; // float
            budget_id: string; // uuid
            date: string; // date
            payee_id: string; // uuid
        }
        export interface CreateUserRequest {
            email: string;
            name: string;
        }
        export interface Payee {
            id: string; // uuid
            name: string;
            user_id: string; // uuid
        }
        export type RepeatingTargetType = "BuildUpTo" | "RequireRepeating";
        export interface Schedule {
            id: string; // uuid
            period: SchedulePeriod;
        }
        export type SchedulePeriod = {
            starting_on: string; // date
            type: "Weekly";
        } | {
            starting_on: string; // date
            type: "Fortnightly";
        } | {
            starting_on: string; // date
            type: "Monthly";
        } | {
            starting_on: string; // date
            type: "Yearly";
        } | {
            every_x_periods: number; // int32
            period: SchedulePeriodType;
            type: "Custom";
        };
        export type SchedulePeriodType = "Weekly" | "Fortnightly" | "Monthly" | "Yearly";
        export interface Transaction {
            amount: number; // float
            bank_account_id: string; // uuid
            budget_id: string; // uuid
            date: string; // date
            id: string; // uuid
            payee_id: string; // uuid
        }
        export interface UpdateBankAccountRequest {
            name: string;
        }
        export interface UpdateBudgetRequest {
            name: string;
            target?: {
                target_amount: number; // float
                type: "OneTime";
            } | {
                repeating_type: RepeatingTargetType;
                schedule: UpdateScheduleRequest;
                target_amount: number; // float
                type: "Repeating";
            };
        }
        export type UpdateBudgetTargetRequest = {
            target_amount: number; // float
            type: "OneTime";
        } | {
            repeating_type: RepeatingTargetType;
            schedule: UpdateScheduleRequest;
            target_amount: number; // float
            type: "Repeating";
        };
        export interface UpdatePayeeRequest {
            name: string;
        }
        export interface UpdateScheduleRequest {
            period: SchedulePeriod;
        }
        export interface UpdateTransactionRequest {
            amount: number; // float
            budget_id: string; // uuid
            date: string; // date
            payee_id: string; // uuid
        }
        export interface User {
            email: string;
            id: string; // uuid
            name: string;
        }
    }
}
declare namespace Paths {
    namespace CreateBankAccount {
        export type RequestBody = Components.Schemas.CreateBankAccountRequest;
        namespace Responses {
            export type $201 = string; // uuid
        }
    }
    namespace CreateBudget {
        export type RequestBody = Components.Schemas.CreateBudgetRequest;
        namespace Responses {
            export type $201 = string; // uuid
        }
    }
    namespace CreatePayee {
        export type RequestBody = Components.Schemas.CreatePayeeRequest;
        namespace Responses {
            export type $201 = string; // uuid
        }
    }
    namespace CreateTransaction {
        namespace Parameters {
            export type BankAccountId = string; // uuid
        }
        export interface PathParameters {
            bankAccountId: Parameters.BankAccountId /* uuid */;
        }
        export type RequestBody = Components.Schemas.CreateTransactionRequest;
        namespace Responses {
            export type $201 = string; // uuid
        }
    }
    namespace CreateUser {
        export type RequestBody = Components.Schemas.CreateUserRequest;
        namespace Responses {
            export type $201 = string; // uuid
        }
    }
    namespace DeleteBankAccount {
        namespace Parameters {
            export type AccountId = string; // uuid
            export type UserId = string; // uuid
        }
        export interface PathParameters {
            accountId: Parameters.AccountId /* uuid */;
        }
        export interface QueryParameters {
            user_id: Parameters.UserId /* uuid */;
        }
        namespace Responses {
            export interface $200 {
            }
        }
    }
    namespace DeleteBudget {
        namespace Parameters {
            export type BudgetId = string; // uuid
        }
        export interface PathParameters {
            budget_id: Parameters.BudgetId /* uuid */;
        }
        namespace Responses {
            export interface $200 {
            }
        }
    }
    namespace DeletePayee {
        namespace Parameters {
            export type PayeeId = string; // uuid
        }
        export interface PathParameters {
            payeeId: Parameters.PayeeId /* uuid */;
        }
        namespace Responses {
            export interface $200 {
            }
        }
    }
    namespace DeleteTransaction {
        namespace Parameters {
            export type TransactionId = string; // uuid
        }
        export interface PathParameters {
            transactionId: Parameters.TransactionId /* uuid */;
        }
        namespace Responses {
            export interface $200 {
            }
        }
    }
    namespace GetBankAccount {
        namespace Parameters {
            export type AccountId = string; // uuid
            export type UserId = string; // uuid
        }
        export interface PathParameters {
            accountId: Parameters.AccountId /* uuid */;
        }
        export interface QueryParameters {
            user_id: Parameters.UserId /* uuid */;
        }
        namespace Responses {
            export type $200 = Components.Schemas.BankAccount;
        }
    }
    namespace GetBankAccounts {
        namespace Parameters {
            export type UserId = string; // uuid
        }
        export interface QueryParameters {
            user_id: Parameters.UserId /* uuid */;
        }
        namespace Responses {
            export type $200 = Components.Schemas.BankAccount[];
        }
    }
    namespace GetBudgets {
        namespace Parameters {
            export type UserId = string; // uuid
        }
        export interface QueryParameters {
            user_id: Parameters.UserId /* uuid */;
        }
        namespace Responses {
            export type $200 = Components.Schemas.Budget[];
        }
    }
    namespace GetPayees {
        namespace Parameters {
            export type UserId = string; // uuid
        }
        export interface QueryParameters {
            user_id: Parameters.UserId /* uuid */;
        }
        namespace Responses {
            export type $200 = Components.Schemas.Payee[];
        }
    }
    namespace GetTransactions {
        namespace Parameters {
            export type BankAccountId = string; // uuid
        }
        export interface PathParameters {
            bankAccountId: Parameters.BankAccountId /* uuid */;
        }
        namespace Responses {
            export type $200 = Components.Schemas.Transaction[];
        }
    }
    namespace GetUser {
        namespace Parameters {
            export type UserId = string; // uuid
        }
        export interface PathParameters {
            userId: Parameters.UserId /* uuid */;
        }
        namespace Responses {
            export type $200 = Components.Schemas.User;
        }
    }
    namespace GetUsers {
        namespace Responses {
            export type $200 = Components.Schemas.User[];
        }
    }
    namespace UpdateBankAccount {
        namespace Parameters {
            export type AccountId = string; // uuid
            export type UserId = string; // uuid
        }
        export interface PathParameters {
            accountId: Parameters.AccountId /* uuid */;
        }
        export interface QueryParameters {
            user_id: Parameters.UserId /* uuid */;
        }
        export type RequestBody = Components.Schemas.UpdateBankAccountRequest;
        namespace Responses {
            export interface $200 {
            }
        }
    }
    namespace UpdateBudget {
        namespace Parameters {
            export type BudgetId = string; // uuid
        }
        export interface PathParameters {
            budget_id: Parameters.BudgetId /* uuid */;
        }
        export type RequestBody = Components.Schemas.UpdateBudgetRequest;
        namespace Responses {
            export interface $200 {
            }
        }
    }
    namespace UpdatePayee {
        namespace Parameters {
            export type PayeeId = string; // uuid
        }
        export interface PathParameters {
            payeeId: Parameters.PayeeId /* uuid */;
        }
        export type RequestBody = Components.Schemas.UpdatePayeeRequest;
        namespace Responses {
            export interface $200 {
            }
        }
    }
    namespace UpdateTransaction {
        namespace Parameters {
            export type BankAccountId = string; // uuid
            export type TransactionId = string; // uuid
        }
        export interface PathParameters {
            bankAccountId: Parameters.BankAccountId /* uuid */;
            transactionId: Parameters.TransactionId /* uuid */;
        }
        export type RequestBody = Components.Schemas.UpdateTransactionRequest;
        namespace Responses {
            export interface $200 {
            }
        }
    }
}

export interface OperationMethods {
  /**
   * getBankAccounts
   */
  'getBankAccounts'(
    parameters?: Parameters<Paths.GetBankAccounts.QueryParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.GetBankAccounts.Responses.$200>
  /**
   * createBankAccount
   */
  'createBankAccount'(
    parameters?: Parameters<UnknownParamsObject> | null,
    data?: Paths.CreateBankAccount.RequestBody,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.CreateBankAccount.Responses.$201>
  /**
   * getBankAccount
   */
  'getBankAccount'(
    parameters?: Parameters<Paths.GetBankAccount.QueryParameters & Paths.GetBankAccount.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.GetBankAccount.Responses.$200>
  /**
   * updateBankAccount
   */
  'updateBankAccount'(
    parameters?: Parameters<Paths.UpdateBankAccount.QueryParameters & Paths.UpdateBankAccount.PathParameters> | null,
    data?: Paths.UpdateBankAccount.RequestBody,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.UpdateBankAccount.Responses.$200>
  /**
   * deleteBankAccount
   */
  'deleteBankAccount'(
    parameters?: Parameters<Paths.DeleteBankAccount.QueryParameters & Paths.DeleteBankAccount.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.DeleteBankAccount.Responses.$200>
  /**
   * getTransactions
   */
  'getTransactions'(
    parameters?: Parameters<Paths.GetTransactions.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.GetTransactions.Responses.$200>
  /**
   * createTransaction
   */
  'createTransaction'(
    parameters?: Parameters<Paths.CreateTransaction.PathParameters> | null,
    data?: Paths.CreateTransaction.RequestBody,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.CreateTransaction.Responses.$201>
  /**
   * getBudgets
   */
  'getBudgets'(
    parameters?: Parameters<Paths.GetBudgets.QueryParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.GetBudgets.Responses.$200>
  /**
   * createBudget
   */
  'createBudget'(
    parameters?: Parameters<UnknownParamsObject> | null,
    data?: Paths.CreateBudget.RequestBody,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.CreateBudget.Responses.$201>
  /**
   * updateBudget
   */
  'updateBudget'(
    parameters?: Parameters<Paths.UpdateBudget.PathParameters> | null,
    data?: Paths.UpdateBudget.RequestBody,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.UpdateBudget.Responses.$200>
  /**
   * deleteBudget
   */
  'deleteBudget'(
    parameters?: Parameters<Paths.DeleteBudget.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.DeleteBudget.Responses.$200>
  /**
   * getPayees
   */
  'getPayees'(
    parameters?: Parameters<Paths.GetPayees.QueryParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.GetPayees.Responses.$200>
  /**
   * createPayee
   */
  'createPayee'(
    parameters?: Parameters<UnknownParamsObject> | null,
    data?: Paths.CreatePayee.RequestBody,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.CreatePayee.Responses.$201>
  /**
   * updatePayee
   */
  'updatePayee'(
    parameters?: Parameters<Paths.UpdatePayee.PathParameters> | null,
    data?: Paths.UpdatePayee.RequestBody,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.UpdatePayee.Responses.$200>
  /**
   * deletePayee
   */
  'deletePayee'(
    parameters?: Parameters<Paths.DeletePayee.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.DeletePayee.Responses.$200>
  /**
   * updateTransaction
   */
  'updateTransaction'(
    parameters?: Parameters<Paths.UpdateTransaction.PathParameters> | null,
    data?: Paths.UpdateTransaction.RequestBody,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.UpdateTransaction.Responses.$200>
  /**
   * deleteTransaction
   */
  'deleteTransaction'(
    parameters?: Parameters<Paths.DeleteTransaction.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.DeleteTransaction.Responses.$200>
  /**
   * getUsers
   */
  'getUsers'(
    parameters?: Parameters<UnknownParamsObject> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.GetUsers.Responses.$200>
  /**
   * createUser
   */
  'createUser'(
    parameters?: Parameters<UnknownParamsObject> | null,
    data?: Paths.CreateUser.RequestBody,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.CreateUser.Responses.$201>
  /**
   * getUser
   */
  'getUser'(
    parameters?: Parameters<Paths.GetUser.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.GetUser.Responses.$200>
}

export interface PathsDictionary {
  ['/api/bank-accounts']: {
    /**
     * getBankAccounts
     */
    'get'(
      parameters?: Parameters<Paths.GetBankAccounts.QueryParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.GetBankAccounts.Responses.$200>
    /**
     * createBankAccount
     */
    'post'(
      parameters?: Parameters<UnknownParamsObject> | null,
      data?: Paths.CreateBankAccount.RequestBody,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.CreateBankAccount.Responses.$201>
  }
  ['/api/bank-accounts/{accountId}']: {
    /**
     * getBankAccount
     */
    'get'(
      parameters?: Parameters<Paths.GetBankAccount.QueryParameters & Paths.GetBankAccount.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.GetBankAccount.Responses.$200>
    /**
     * updateBankAccount
     */
    'put'(
      parameters?: Parameters<Paths.UpdateBankAccount.QueryParameters & Paths.UpdateBankAccount.PathParameters> | null,
      data?: Paths.UpdateBankAccount.RequestBody,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.UpdateBankAccount.Responses.$200>
    /**
     * deleteBankAccount
     */
    'delete'(
      parameters?: Parameters<Paths.DeleteBankAccount.QueryParameters & Paths.DeleteBankAccount.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.DeleteBankAccount.Responses.$200>
  }
  ['/api/bank-accounts/{bankAccountId}/transactions']: {
    /**
     * getTransactions
     */
    'get'(
      parameters?: Parameters<Paths.GetTransactions.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.GetTransactions.Responses.$200>
    /**
     * createTransaction
     */
    'post'(
      parameters?: Parameters<Paths.CreateTransaction.PathParameters> | null,
      data?: Paths.CreateTransaction.RequestBody,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.CreateTransaction.Responses.$201>
  }
  ['/api/budgets']: {
    /**
     * getBudgets
     */
    'get'(
      parameters?: Parameters<Paths.GetBudgets.QueryParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.GetBudgets.Responses.$200>
    /**
     * createBudget
     */
    'post'(
      parameters?: Parameters<UnknownParamsObject> | null,
      data?: Paths.CreateBudget.RequestBody,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.CreateBudget.Responses.$201>
  }
  ['/api/budgets/{budget_id}']: {
    /**
     * updateBudget
     */
    'put'(
      parameters?: Parameters<Paths.UpdateBudget.PathParameters> | null,
      data?: Paths.UpdateBudget.RequestBody,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.UpdateBudget.Responses.$200>
    /**
     * deleteBudget
     */
    'delete'(
      parameters?: Parameters<Paths.DeleteBudget.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.DeleteBudget.Responses.$200>
  }
  ['/api/payees']: {
    /**
     * getPayees
     */
    'get'(
      parameters?: Parameters<Paths.GetPayees.QueryParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.GetPayees.Responses.$200>
    /**
     * createPayee
     */
    'post'(
      parameters?: Parameters<UnknownParamsObject> | null,
      data?: Paths.CreatePayee.RequestBody,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.CreatePayee.Responses.$201>
  }
  ['/api/payees/{payeeId}']: {
    /**
     * updatePayee
     */
    'put'(
      parameters?: Parameters<Paths.UpdatePayee.PathParameters> | null,
      data?: Paths.UpdatePayee.RequestBody,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.UpdatePayee.Responses.$200>
    /**
     * deletePayee
     */
    'delete'(
      parameters?: Parameters<Paths.DeletePayee.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.DeletePayee.Responses.$200>
  }
  ['/api/transactions/{transactionId}']: {
    /**
     * updateTransaction
     */
    'put'(
      parameters?: Parameters<Paths.UpdateTransaction.PathParameters> | null,
      data?: Paths.UpdateTransaction.RequestBody,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.UpdateTransaction.Responses.$200>
    /**
     * deleteTransaction
     */
    'delete'(
      parameters?: Parameters<Paths.DeleteTransaction.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.DeleteTransaction.Responses.$200>
  }
  ['/api/users']: {
    /**
     * getUsers
     */
    'get'(
      parameters?: Parameters<UnknownParamsObject> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.GetUsers.Responses.$200>
    /**
     * createUser
     */
    'post'(
      parameters?: Parameters<UnknownParamsObject> | null,
      data?: Paths.CreateUser.RequestBody,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.CreateUser.Responses.$201>
  }
  ['/api/users/{userId}']: {
    /**
     * getUser
     */
    'get'(
      parameters?: Parameters<Paths.GetUser.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.GetUser.Responses.$200>
  }
}

export type Client = OpenAPIClient<OperationMethods, PathsDictionary>

export type BankAccount = Components.Schemas.BankAccount;
export type Budget = Components.Schemas.Budget;
export type BudgetTarget = Components.Schemas.BudgetTarget;
export type CreateBankAccountRequest = Components.Schemas.CreateBankAccountRequest;
export type CreateBudgetRequest = Components.Schemas.CreateBudgetRequest;
export type CreateBudgetTargetRequest = Components.Schemas.CreateBudgetTargetRequest;
export type CreatePayeeRequest = Components.Schemas.CreatePayeeRequest;
export type CreateScheduleRequest = Components.Schemas.CreateScheduleRequest;
export type CreateTransactionRequest = Components.Schemas.CreateTransactionRequest;
export type CreateUserRequest = Components.Schemas.CreateUserRequest;
export type Payee = Components.Schemas.Payee;
export type RepeatingTargetType = Components.Schemas.RepeatingTargetType;
export type Schedule = Components.Schemas.Schedule;
export type SchedulePeriod = Components.Schemas.SchedulePeriod;
export type SchedulePeriodType = Components.Schemas.SchedulePeriodType;
export type Transaction = Components.Schemas.Transaction;
export type UpdateBankAccountRequest = Components.Schemas.UpdateBankAccountRequest;
export type UpdateBudgetRequest = Components.Schemas.UpdateBudgetRequest;
export type UpdateBudgetTargetRequest = Components.Schemas.UpdateBudgetTargetRequest;
export type UpdatePayeeRequest = Components.Schemas.UpdatePayeeRequest;
export type UpdateScheduleRequest = Components.Schemas.UpdateScheduleRequest;
export type UpdateTransactionRequest = Components.Schemas.UpdateTransactionRequest;
export type User = Components.Schemas.User;
