CREATE TABLE Payees(
    `id` CHAR(32) NOT NULL PRIMARY KEY,
    `name` VARCHAR(255) NOT NULL
);

CREATE TABLE Transactions(
    `id` CHAR(32) NOT NULL PRIMARY KEY,
    `payee_id` CHAR(32) NOT NULL,
    `amount_dollars` INT(10) NOT NULl,
    `amount_cents` INT(2) NOT NULL,
    FOREIGN KEY (`payee_id`) REFERENCES Payees(`id`)
);
