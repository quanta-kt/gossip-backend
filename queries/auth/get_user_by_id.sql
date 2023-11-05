SELECT
    id, username, email, password_hash, is_verified
FROM gossip_user
WHERE id = $1
