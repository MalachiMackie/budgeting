ALTER TABLE BudgetAssignments
ADD COLUMN `assignment_type` VARCHAR(32) NOT NULL,
ADD COLUMN `from_budget_id` CHAR(32) NULL,
ADD COLUMN `from_transaction_id` CHAR(32) NULL,
ADD CONSTRAINT `FK_BudgetAssignments_FromBudgetId` FOREIGN KEY (`from_budget_id`) REFERENCES Budgets(`id`),
ADD CONSTRAINT `FK_BudgetAssignments_Transaction` FOREIGN KEY (`from_transaction_id`) REFERENCES Transactions(`id`);