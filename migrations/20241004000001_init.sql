CREATE TABLE users
(
    uid                 TEXT NOT NULL,
    email               TEXT NOT NULL,
    pass_hash           TEXT,
    created_date        TEXT NOT NULL,
    last_pass_change    TEXT NOT NULL,

    set_pass_token      TEXT,
    set_pass_token_date TEXT,

    CONSTRAINT users_pk PRIMARY KEY (uid),
    CONSTRAINT idx_users_email UNIQUE (email)
) strict;

INSERT INTO users (uid, email, pass_hash, created_date, last_pass_change)
VALUES ('8b90ed88-53f1-4251-a1e4-07f4b11eef87', 'sieciech.czajka@golem.network',
        '64e3da154eec6064860f534c2676230731c8322e', '2024-10-04 00:00:01',
        '2024-10-04 00:00:01');