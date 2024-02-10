CREATE TABLE transactions
(
    client_id    integer                  not null,
    valor        integer                  not null,
    tipo         varchar(1)               not null,
    descricao    varchar(10)              not null,
    realizada_em timestamp with time zone not null,

    constraint fk_client_id foreign key (client_id) references clients (id)
);
