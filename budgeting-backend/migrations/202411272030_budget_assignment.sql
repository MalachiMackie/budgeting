CREATE TABLE BudgetAssignments (
    id CHAR(32) NOT NULL PRIMARY KEY,
    amount DECIMAL(10, 2) NOT NULL,
    budget_id CHAR(32) NOT NULL,
    `date` DATE NOT NULL,
    FOREIGN KEY (budget_id) REFERENCES Budgets(id)
);