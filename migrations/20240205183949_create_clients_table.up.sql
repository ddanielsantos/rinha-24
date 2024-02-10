CREATE TABLE clients
(
    id           integer primary key generated always as identity not null,
    name         varchar(50)                                      not null,
    credit_limit integer                                          not null,
    balance      integer                                          not null default 0
);

CREATE INDEX id_index ON clients (id);

DO
$$
    BEGIN
        INSERT INTO clients (name, credit_limit)
        VALUES ('o barato sai caro', 1000 * 100),
               ('zan corp ltda', 800 * 100),
               ('les cruders', 10000 * 100),
               ('padaria joia de cocaia', 100000 * 100),
               ('kid mais', 5000 * 100);
    END;
$$