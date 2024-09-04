import { useEffect, useState } from "react"

type Transaction = {
    id: string;
    payee_id: string;
    amount_dollars: number;
    amount_cents: number;
};

export default function TransactionList(): JSX.Element {
    const [transactions, setTransactions] = useState<Transaction[]>([]);

    useEffect(() => {
        let promise = async () => {
            let result = await fetch("http://localhost:3000/api/transactions");
            let json = await result.json()
            setTransactions(json as Transaction[]);
        }

        void promise();
    }, []);

    return <>{transactions.map((x) => <div>{x.id}: ${x.amount_dollars}.{x.amount_cents.toFixed(2)}</div>)}</>
}