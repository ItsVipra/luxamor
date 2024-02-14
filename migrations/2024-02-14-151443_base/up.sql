-- Your SQL goes here
CREATE TABLE `users`(
	`id` INTEGER PRIMARY KEY,
	`name` TEXT NOT NULL,
	`link` TEXT,
	`enabled` BOOL NOT NULL
);

CREATE TABLE `pings`(
	`id` INTEGER PRIMARY KEY,
	`timestamp` TIMESTAMP,
	`origin` TEXT,
	`color` TEXT NOT NULL
);

