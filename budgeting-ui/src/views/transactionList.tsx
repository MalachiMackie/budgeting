import { Button, DataTableSkeleton, DatePicker, DatePickerInput, Dropdown, NumberInput, Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@carbon/react";
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
        return <DataTableSkeleton />
    }

    const headers  =[{key: 'time', header: "Time"}, {key: 'amount', header: "Amount"}, {key: 'payee', header: 'Payee'}];

    const saveTransaction = async (x: CreateTransactionRequest) => {
        await fetch("http://localhost:3000/api/transactions", {method: 'POST', body: JSON.stringify(x), headers: {"Content-Type": "application/json"}});
    }

    const sortTransactions = (a: Transaction, b: Transaction): number => {
        // todo: within the same day, compare by amount
        return a.time.localeCompare(b.time);
    }

    return <Table size="sm">
        <TableHead>
            <TableRow>
                {headers.map(x => <TableHeader key={x.key} id={x.key}>{x.header}</TableHeader>)}
            </TableRow>
        </TableHead>
        <TableBody>
            <NewTransactionRow onComplete={() => refresh()} save={saveTransaction} payees={payees} />
            {[...transactions].sort(sortTransactions).map(x => <TableRow key={x.id}>
                <TableCell>{new Date(x.time).toLocaleDateString()}</TableCell>
                <TableCell>${(x.amount_dollars + ((x.amount_cents * Math.sign(x.amount_dollars)) / 100)).toFixed(2)}</TableCell>
                <TableCell>{payeesMap.get(x.payee_id)!.name}</TableCell>
            </TableRow>)}
        </TableBody>
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

    const dropdownItems = payees.map(x => ({id: x.id, value: x.id, label: x.name}));

    return <TableRow>
        <TableCell>
            <DatePicker onChange={x => x.length == 1 && setDate(x[0])} value={date} datePickerType="single" >
                <DatePickerInput size="sm" id={"date"} labelText={undefined} />
            </DatePicker>
        </TableCell>
        <TableCell>
            <NumberInput value={amount} onChange={(_, {value}) => typeof value === "number" && setAmount(value)} size="sm" id={"amount"} />
        </TableCell>
        <TableCell>
            <div style={{display: 'flex', flexDirection: 'column'}}>
                <Dropdown onChange={x => x.selectedItem && setPayeeId(x.selectedItem.id)} initialSelectedItem={dropdownItems.find(x => x.id === payeeId)} itemToString={x => x?.label ?? ""} size="sm" id={"payee"} items={dropdownItems} label={""} />
                <Button onClick={handleSaveClick}>Save</Button>
            </div>
        </TableCell>
    </TableRow>;
}