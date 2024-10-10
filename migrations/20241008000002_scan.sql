
CREATE TABLE scan
(
    address TEXT NOT NULL,
    first_block_number INT NOT NULL,
    first_block_timestamp TEXT NOT NULL,
    next_block_number INT NOT NULL,
    next_block_timestamp TEXT NOT NULL,

    CONSTRAINT scan_info_pk PRIMARY KEY (address)
) strict;

CREATE TABLE block
(
    address TEXT NOT NULL,
    block_number INT NOT NULL,
    timestamp TEXT NOT NULL,
    balance TEXT NOT NULL,
    balance_diff TEXT NOT NULL,
    updated TEXT NOT NULL,
    block_miner TEXT NOT NULL,
    consensus_reward TEXT NOT NULL,
    mev_reward TEXT NOT NULL,
    block_reward TEXT NOT NULL,
    amount_incoming TEXT NOT NULL,
    amount_outgoing TEXT NOT NULL,

    CONSTRAINT block_pk PRIMARY KEY (address, block_number),
    CONSTRAINT block_scan_info_fk FOREIGN KEY (address)
        REFERENCES scan (address)
        ON DELETE CASCADE
) strict;

CREATE TABLE tx
(
    address TEXT NOT NULL,
    tx_hash TEXT NOT NULL,
    block_number INT NOT NULL,
    block_index INT NOT NULL,
    gas_used TEXT NOT NULL,

    CONSTRAINT tx_pk PRIMARY KEY (address, tx_hash, block_number, block_index),
    CONSTRAINT tx_block_fk FOREIGN KEY (address, block_number)
        REFERENCES block (address, block_number)
        ON DELETE CASCADE
) strict;

CREATE TABLE tx_trace
(
    address TEXT NOT NULL,
    tx_hash TEXT NOT NULL,
    block_number INT NOT NULL,
    block_index INT NOT NULL,
    trace_index INT NOT NULL,
    from_addr TEXT NOT NULL,
    to_addr TEXT NOT NULL,
    value TEXT NOT NULL,
    gas_used TEXT NOT NULL,

    CONSTRAINT tx_trace_pk PRIMARY KEY (address, tx_hash, block_number, block_index, trace_index),
    CONSTRAINT tx_trace_tx_fk FOREIGN KEY (address, tx_hash, block_number, block_index)
        REFERENCES tx (address, tx_hash, block_number, block_index)
        ON DELETE CASCADE
) strict;
