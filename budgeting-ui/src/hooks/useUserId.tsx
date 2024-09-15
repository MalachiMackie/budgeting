import { createContext, useContext } from "react";

export const UserIdContext = createContext<{ userId: string }>({ userId: "" });

export function useUserId(): string {
  return useContext(UserIdContext).userId;
}
