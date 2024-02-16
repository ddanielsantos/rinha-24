CREATE
    OR REPLACE FUNCTION ALTER_BALANCE(
    client_id int,
    operation_value int,
    operation varchar(1),
    description varchar(10)
)
    RETURNS TABLE
            (
                success     int,
                new_balance int
            )
AS
$$
DECLARE
    balance     int;
    new_balance int;
BEGIN
    IF operation = 'c' THEN
        select c.balance into balance from clients c where id = client_id;

        new_balance = balance + operation_value;

        update clients set balance = new_balance where id = client_id;

        insert into transactions (client_id, value, type, description, created_at)
        values (client_id, new_balance, operation, description, now());

        return QUERY SELECT 1, new_balance;
        return;
    END IF;
END;

$$ LANGUAGE plpgsql;