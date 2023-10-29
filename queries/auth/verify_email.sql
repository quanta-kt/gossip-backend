WITH
    update_result AS (
        UPDATE gossip_user
        SET is_verified = TRUE
        WHERE email = $1
        RETURNING id
    ),
    _ AS (
        DELETE FROM pending_email_verification
        WHERE user_id IN (SELECT id FROM update_result)
    )

SELECT id, username, email, password_hash, is_verified
FROM gossip_user
WHERE id IN (SELECT id FROM update_result)