CREATE TABLE Schedules(
    `id` CHAR(32) NOT NULL PRIMARY KEY,
    `period_type` VARCHAR(32) NOT NULL,
    `period_starting_on` DATE NULL,
    `custom_period_type` VARCHAR(32) NULL,
    `custom_period_every_count` INT NULL
);