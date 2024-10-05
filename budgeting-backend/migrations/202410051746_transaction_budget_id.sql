ALTER TABLE Transactions
ADD COLUMN `budget_id` CHAR(32) NOT NULL,
ADD CONSTRAINT `FK_Transactions_Budget` FOREIGN KEY (`budget_id`) REFERENCES Budgets(`id`)