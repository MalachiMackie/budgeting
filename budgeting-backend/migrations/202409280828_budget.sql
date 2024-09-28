CREATE TABLE Budgets(
    `id` CHAR(32) NOT NULL PRIMARY KEY,
    `name` VARCHAR(255) NOT NULL,
    `target_type` VARCHAR(32) NULL,
    `repeating_target_type` VARCHAR(32) NULL,
    `target_amount` DECIMAL(10, 2) NULL,
    `target_schedule_id` CHAR(32) NULL,
    `user_id` CHAR(32) NOT NULL,
    CONSTRAINT FK_budget_schedule_id Foreign Key(`target_schedule_id`) REFERENCES Schedules (`id`),
    CONSTRAINT FK_budget_user_id Foreign Key(`user_id`) REFERENCES Users (`id`)
)