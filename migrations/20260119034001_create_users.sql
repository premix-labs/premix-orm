-- up
CREATE TABLE premix_migration_test_users (
    id INTEGER PRIMARY KEY,
    username TEXT NOT NULL,
    email TEXT NOT NULL
);

-- down
DROP TABLE premix_migration_test_users;
