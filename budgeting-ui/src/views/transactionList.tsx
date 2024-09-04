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

    useEffect(() => {
        let loadTransactions = async () => {
            let result = await fetch("http://localhost:3000/api/transactions");
            let json = await result.json();
            setTransactions(json as Transaction[]);
        }
        let loadPayees = async () => {
            let result = await fetch("http://localhost:3000/api/payees");
            let json = await result.json();
            setPayees(json as Payee[]);
        }

        void loadTransactions();
        void loadPayees();
    }, []);

    let payeesMap = new Map(payees.map(x => [x.id, x]))

    return <>{transactions.map((x) => <div>{x.id}: ${x.amount_dollars}.{x.amount_cents.toFixed(2)} to {payeesMap.get(x.payee_id)?.name}</div>)}</>
}