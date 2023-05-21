-- sqlx migrate add seed_user

-- Add migration script here
INSERT INTO users
VALUES (
        '9c282196-1734-41d8-be03-22469c1d0546',
        'admin',
        '$argon2id$v=19$m=19456,t=2,p=1$Ti2FKTqKZ6KVnlmNO2aNpw$+/CfTSllVJuWE5y0HV4V8Y7LkTM8A8j0GyWgjlAnIRk'
        )