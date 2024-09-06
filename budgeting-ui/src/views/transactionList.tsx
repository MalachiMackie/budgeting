import { Button, NumberInput, Select, Table } from "@mantine/core";
import { DatePickerInput } from "@mantine/dates";
import { useEffect, useState } from "react"

type Transaction = {
    id: string;
    payee_id: string;
    amount_dollars: number;
    amount_cents: number;
    time: string;
};

type Payee = {
    id: string;
    name: string;
}

export default function TransactionList(): JSX.Element {
    const [transactions, setTransactions] = useState<Transaction[]>([]);
    const [payees, setPayees] = useState<Payee[]>([]);
    const [loadingState, setLoading] = useState<{transactions: boolean; payees: boolean}>({transactions: false, payees: false});
    const loading = loadingState.transactions || loadingState.payees;
    const [refreshCount, setRefreshCount] = useState(0);
    const refresh = () => setRefreshCount(x => x + 1);

    useEffect(() => {
        let loadTransactions = async () => {
            setLoading(x => ({...x, transactions: true}));
            let result = await fetch("http://localhost:3000/api/transactions");
            let json = await result.json();
            setTransactions(json as Transaction[]);
            setLoading(x => ({...x, transactions: false}));
        }
        let loadPayees = async () => {
            setLoading(x => ({...x, payees: true}));
            let result = await fetch("http://localhost:3000/api/payees");
            let json = await result.json();
            setPayees(json as Payee[]);
            setLoading(x => ({...x, payees: false}));
        }

        void loadTransactions();
        void loadPayees();
    }, [refreshCount]);

    let payeesMap = new Map(payees.map(x => [x.id, x]))

    if (loading) {
        return <>Loading...</>
    }

    const headers  =[{key: 'time', header: "Time"}, {key: 'amount', header: "Amount"}, {key: 'payee', header: 'Payee'}];

    const saveTransaction = async (x: CreateTransactionRequest) => {
        await fetch("http://localhost:3000/api/transactions", {method: 'POST', body: JSON.stringify(x), headers: {"Content-Type": "application/json"}});
    }

    const compareTransactions = (a: Transaction, b: Transaction): number => {
        // todo: within the same day, compare by amount

        // sort descending time
        return b.time.localeCompare(a.time);
    }

    return <Table>
        <Table.Thead>
            <Table.Tr>
                {headers.map(x => <Table.Th key={x.key}>{x.header}</Table.Th>)}
            </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
            <NewTransactionRow onComplete={() => refresh()} save={saveTransaction} payees={payees} />
            {[...transactions].sort(compareTransactions).map(x => <Table.Tr>
                <Table.Td>{new Date(x.time).toDateString()}</Table.Td>
                <Table.Td>${(x.amount_dollars + ((x.amount_cents * Math.sign(x.amount_dollars)) / 100)).toFixed(2)}</Table.Td>
                <Table.Td>{payeesMap.get(x.payee_id)?.name}</Table.Td>
            </Table.Tr>)}
        </Table.Tbody>
    </Table>
}

type CreateTransactionRequest = {
    payee_id: string,
    amount_dollars: number,
    amount_cents: number,
    time: string
};

function NewTransactionRow({payees, save, onComplete}: {payees: Payee[], save: (x: CreateTransactionRequest) => Promise<void>, onComplete: () => void}): JSX.Element {
    const [date, setDate] = useState(new Date());
    const [amount, setAmount] = useState<number | undefined>(undefined);
    const [payeeId, setPayeeId] = useState<string | undefined>(undefined)
    const handleSaveClick = async () => {
        if (!payeeId) {
            alert("Cannot save transaction without a payee");
            return;
        }
        if (typeof amount === "undefined") {
            alert("Cannot save transaction without a transaction amount");
            return;
        }
        // todo: do this better maybe?
        const amountDollars = Math.trunc(amount);
        const amountCents = Math.trunc(Math.abs((amount - amountDollars)) * 100);
        try {
            await save({
                time: date.toISOString(),
                payee_id: payeeId,
                amount_dollars: amountDollars,
                amount_cents: amountCents
            });
            onComplete();
            setDate(new Date());
            setAmount(undefined);
            setPayeeId(undefined);
        } catch {
            // handle error
        }
    }

    return <><Table.Tr style={{borderBottom: 0}}>
        <Table.Td>
            <DatePickerInput value={date} onChange={x => x && setDate(x)} />
        </Table.Td>
        <Table.Td>
            <NumberInput value={amount} onChange={value => typeof value === "number" && setAmount(value)} />
        </Table.Td>
        <Table.Td>
            <div style={{display: 'flex', flexDirection: 'column'}}>
                <Select onChange={x => x && setPayeeId(x)} data={payees.map(x => ({value: x.id, label: x.name}))} value={payeeId} />
            </div>
        </Table.Td>
    </Table.Tr>
    <Table.Tr>
        <Table.Td colSpan={3}>
                <Button onClick={handleSaveClick}>Save</Button>
        </Table.Td>
    </Table.Tr>
    </>
}