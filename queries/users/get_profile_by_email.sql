SELECT
    id, username, bio
FROM gossip_user
WHERE email = $1
