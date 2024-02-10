CREATE TABLE transactions
(
    client_id   integer                  not null,
    value       integer                  not null,
    type        varchar(1)               not null,
    description varchar(10)              not null,
    created_at  timestamp with time zone not null,

    constraint fk_client_id foreign key (client_id) references clients (id)
);
