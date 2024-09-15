import { useUserId } from "../hooks/useUserId";
import { BankAccountList } from "../views/BankAccountList";

export function Accounts(): JSX.Element {
  const userId = useUserId();
  return <BankAccountList userId={userId} />;
}
