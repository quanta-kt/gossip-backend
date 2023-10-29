SELECT user_id, code
FROM pending_email_verification
JOIN gossip_user ON
    gossip_user.id = pending_email_verification.user_id
WHERE
    gossip_user.email = $1 AND gossip_user.is_verified = FALSE
