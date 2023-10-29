SELECT
    id, username, bio
FROM gossip_user
WHERE id = $1 AND is_verified = TRUE
