import { MantineProvider } from "@mantine/core";
import "@mantine/core/styles.css";
import "@mantine/dates/styles.css";
import {
  QueryClient,
  QueryClientProvider,
  useQueryClient,
} from "@tanstack/react-query";
import OpenAPIClientAxios, { Document } from "openapi-client-axios";
import { createContext, useContext, useEffect, useState } from "react";
import {
  createBrowserRouter,
  Params,
  RouteObject,
  RouterProvider,
  useRouteError,
} from "react-router-dom";
import apiDefinition from "../../api-doc.json";
import { Client } from "./api/client";
import { UserIdContext, useUserId } from "./hooks/useUserId";
import { AccountPage, createAccountLoader } from "./routes/AccountPage";
import { AccountsPage, createAccountsLoader } from "./routes/AccountsPage";
import { BudgetsPage, createBudgetsLoader } from "./routes/BudgetsPage";
import { createPayeesLoader, PayeesPage } from "./routes/PayeesPage";
import { createRootLoader, Root } from "./routes/Root";
import {
  createUserAccountLoader,
  UserAccountPage,
} from "./routes/UserAccountPage";

export const BudgetingApiContext = createContext<Client>(null!);

export function useBudgetingApi(): Client {
  return useContext(BudgetingApiContext);
}

const queryClient = new QueryClient();

const apiClient = new OpenAPIClientAxios({
  definition: {
    ...apiDefinition,
    servers: [{ url: "http://localhost:3000" }],
  } as unknown as Document,
});

function App() {
  const [user, setUser] = useState<string | null>(null);
  // store api in an object so that the setApi doesn't think we're trying to do a prev callback
  const [{ client: api }, setApi] = useState<{ client: Client | null }>({
    client: null,
  });

  useEffect(() => {
    apiClient.getClient<Client>().then((x) => {
      setApi({ client: x });
    });
  }, []);

  // for now, just load the first user
  useEffect(() => {
    if (!api) {
      return;
    }
    let load = async () => {
      let users = await api.getUsers();
      if (users.data.length == 0) {
        throw new Error("No users!");
      }
      setUser(users.data[0].id);
    };
    void load();
  }, [api]);

  return (
    <MantineProvider defaultColorScheme="dark">
      {api && (
        <BudgetingApiContext.Provider value={api}>
          <QueryClientProvider client={queryClient}>
            {user !== null && (
              <UserIdContext.Provider value={{ userId: user }}>
                <BudgetingRouterWrapper />
              </UserIdContext.Provider>
            )}
          </QueryClientProvider>
        </BudgetingApiContext.Provider>
      )}
    </MantineProvider>
  );
}

// temporary until actually figure out user id
export function BudgetingRouterWrapper() {
  const api = useBudgetingApi();
  const userId = useUserId();
  const queryClient = useQueryClient();

  const router = createBrowserRouter([
    {
      path: "/",
      element: <Root />,
      loader: createRootLoader(api, queryClient, userId),
      errorElement: <ErrorComponent />,
      children: [
        {
          path: "/accounts/:accountId",
          element: <AccountPage />,
          loader: createAccountLoader(api, queryClient, userId),
        },
        {
          path: "/accounts",
          element: <AccountsPage />,
          loader: createAccountsLoader(api, queryClient, userId),
        },
        {
          path: "/budgets",
          element: <BudgetsPage />,
          loader: createBudgetsLoader(api, queryClient, userId),
        },
        {
          path: "/payees",
          element: <PayeesPage />,
          loader: createPayeesLoader(api, queryClient, userId),
        },
        {
          path: "/user-account",
          element: <UserAccountPage />,
          loader: createUserAccountLoader(api, queryClient, userId),
        },
      ],
    },
  ] satisfies MaybeHasLoader[]);

  return <RouterProvider router={router} />;
}

// RouteObject allows the loader to return anything, but at runtime,
// it validates that it must return something or null.
// so instead, make a type that is what router expects
type Loader =
  | (() => Promise<{} | null>)
  | ((args: { params: Params }) => Promise<{} | null>)
  | undefined;

type MaybeHasLoader = RouteObject & {
  loader?: Loader;
  children?: MaybeHasLoader[];
};

export default App;

function ErrorComponent(): JSX.Element {
  const error = useRouteError();

  return (
    <>
      <span>Something went wrong 😱</span>
      <code>{JSON.stringify(error)}</code>
    </>
  );
}
