
WITH insert_result AS(
    INSERT INTO gossip_user (email, password_hash, username)
    VALUES ($1, $2, $3)
 
    -- Account creation is idempotent for unverified accounts,
    -- if the email is taken, but the account is not verified, the creation should pass.
    -- when this happens, we update the password and resend the OTP.
    ON CONFLICT (email)
        DO UPDATE
        SET
            password_hash = $2,
            username = $3
        WHERE gossip_user.is_verified = FALSE

    RETURNING id
)

INSERT INTO pending_email_verification (user_id, code)
SELECT id, $4
FROM insert_result
ON CONFLICT (user_id)
    DO UPDATE
    SET code = $4
RETURNING user_id 