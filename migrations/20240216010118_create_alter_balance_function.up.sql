CREATE
    OR REPLACE FUNCTION ALTER_BALANCE(
    client_id INT,
    operation_value INT,
    operation VARCHAR(1),
    description VARCHAR(10)
)
    RETURNS TABLE
            (
                success     INT,
                new_balance INT
            )
AS
$$
DECLARE
    client     clients%ROWTYPE;
    new_balance INT;
BEGIN
    IF operation = 'c' THEN
        SELECT c.* INTO client FROM clients c WHERE id = client_id;

        new_balance = client.balance + operation_value;

        UPDATE clients SET balance = new_balance WHERE id = client_id;

        INSERT INTO transactions (client_id, value, type, description, created_at)
        VALUES (client_id, new_balance, operation, description, now());

        RETURN QUERY SELECT 1, new_balance;
        RETURN;
    END IF;

    IF operation = 'd' THEN
        SELECT c.* INTO client FROM clients c WHERE id = client_id;

        new_balance = client.balance - operation_value;

        IF new_balance < (client.credit_limit * -1) THEN
            RETURN QUERY SELECT -2, 0;
            RETURN;
        END IF;

        UPDATE clients SET balance = new_balance WHERE id = client_id;

        INSERT INTO transactions (client_id, value, type, description, created_at)
        VALUES (client_id, new_balance, operation, description, now());

        RETURN QUERY SELECT 1, new_balance;
        RETURN;
    END IF;

END;

$$ LANGUAGE plpgsql;