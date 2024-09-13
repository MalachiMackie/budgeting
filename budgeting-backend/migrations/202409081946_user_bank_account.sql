CREATE TABLE Users(
    `id` CHAR(32) NOT NULL PRIMARY KEY,
    `name` VARCHAR(255) NOT NULL,
    `email` VARCHAR(255) NOT NULL
);

CREATE TABLE BankAccounts(
    `id` CHAR(32) NOT NULL PRIMARY KEY,
    `name` VARCHAR(255) NOT NULL,
    `initial_amount_dollars` INT(10) NOT NULL,
    `initial_amount_cents` INT(2) NOT NULL,
    `user_id` CHAR(32) NOT NULL,
    CONSTRAINT fk_bank_accounts_user FOREIGN KEY (`user_id`) REFERENCES Users(`id`)
);

ALTER TABLE Transactions
ADD COLUMN `bank_account_id` CHAR(32) NOT NULL;

ALTER TABLE Transactions
ADD CONSTRAINT fk_transactions_bank_account FOREIGN KEY (`bank_account_id`) REFERENCES BankAccounts(`id`);

ALTER TABLE Payees
ADD COLUMN `user_id` CHAR(32) NOT NULL;

ALTER TABLE Payees
ADD CONSTRAINT fk_payees_user FOREIGN KEY (`user_id`) REFERENCES Users(`id`);
