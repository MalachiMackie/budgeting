import { MantineProvider } from '@mantine/core';
import TransactionList from './views/transactionList'
import '@mantine/core/styles.css';
import '@mantine/dates/styles.css'

function App() {
  return (
    <MantineProvider>
      <TransactionList />
    </MantineProvider>
  )
}

export default App
