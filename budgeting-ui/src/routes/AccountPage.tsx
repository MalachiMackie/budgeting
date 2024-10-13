import { Button, Flex, Title } from "@mantine/core";
import { IconTrash } from "@tabler/icons-react";
import {
  QueryClient,
  queryOptions,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { useState } from "react";
import { Params, useNavigate, useParams } from "react-router-dom";
import { BudgetingApi } from "../api/budgetingApi";
import { useBudgetingApi } from "../App";
import { useUserId } from "../hooks/useUserId";
import { queryKeys } from "../queryKeys";
import { DeleteAccountModal } from "../views/DeleteAccountModal";
import { TransactionList } from "../views/TransactionList";

export function AccountPage(): JSX.Element {
  const userId = useUserId();
  const api = useBudgetingApi();
  const params = useParams();
  const navigate = useNavigate();

  const [showDeleteAccount, setShowDeleteAccount] = useState(false);

  const { data: bankAccount } = useSuspenseQuery(
    createQueryOptions(api, params.accountId!, userId)
  );

  const handleAccountDeleted = () => {
    navigate("/accounts");
  };

  return (
    <>
      <Flex>
        <Title>{bankAccount.name} - Transactions</Title>
        <Button
          color="red"
          ml={"auto"}
          onClick={() => setShowDeleteAccount(true)}
        >
          <IconTrash />
        </Button>
      </Flex>
      <span>
        Balance: {Math.sign(bankAccount.balance) === -1 && "-"}$
        {Math.abs(bankAccount.balance).toFixed(2)}
      </span>
      <TransactionList bankAccountId={bankAccount.id} userId={userId} />
      {showDeleteAccount && (
        <DeleteAccountModal
          bankAccount={bankAccount}
          onCancel={() => setShowDeleteAccount(false)}
          onDelete={handleAccountDeleted}
        />
      )}
    </>
  );
}

function createQueryOptions(
  api: BudgetingApi,
  accountId: string,
  userId: string
) {
  return queryOptions({
    queryKey: queryKeys.bankAccounts.fetchSingle(accountId),
    queryFn: () => api.getBankAccount(accountId, userId),
  });
}

export function createAccountLoader(
  api: BudgetingApi,
  queryClient: QueryClient,
  userId: string
) {
  return async ({ params }: { params: Params }) => {
    const accountId = params.accountId!;
    await queryClient.ensureQueryData(
      createQueryOptions(api, accountId, userId)
    );
    return null;
  };
}
