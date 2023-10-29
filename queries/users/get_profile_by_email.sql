SELECT
    id, username, bio
FROM gossip_user
WHERE email = $1 AND is_verified = TRUE
