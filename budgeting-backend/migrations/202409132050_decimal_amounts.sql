ALTER TABLE Transactions
ADD COLUMN amount DECIMAL(10, 2) NOT NULL DEFAULT 0,
DROP COLUMN amount_dollars,
DROP COLUMN amount_cents;

ALTER TABLE BankAccounts
ADD COLUMN initial_amount DECIMAL(10, 2) NOT NULL DEFAULT 0,
DROP COLUMN initial_amount_dollars,
DROP COLUMN initial_amount_cents;