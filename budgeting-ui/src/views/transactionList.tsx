import { DataTableSkeleton, Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@carbon/react";
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
    }, []);

    let payeesMap = new Map(payees.map(x => [x.id, x]))

    if (loading) {
        return <DataTableSkeleton />
    }

    const headers  =[{key: 'time', header: "Time"}, {key: 'amount', header: "Amount"}, {key: 'payee', header: 'Payee'}];

    return <Table size="sm">
        <TableHead>
            <TableRow>
                {headers.map(x => <TableHeader key={x.key} id={x.key}>{x.header}</TableHeader>)}
            </TableRow>
        </TableHead>
        <TableBody>
            {transactions.map(x => <TableRow key={x.id}>
                <TableCell>{new Date(x.time).toLocaleDateString()}</TableCell>
                <TableCell>${(x.amount_dollars + ((x.amount_cents * Math.sign(x.amount_dollars)) / 100)).toFixed(2)}</TableCell>
                <TableCell>{payeesMap.get(x.payee_id)!.name}</TableCell>
            </TableRow>)}
        </TableBody>
    </Table>
}
