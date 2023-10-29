SELECT
    id, username, bio
FROM gossip_user
WHERE id = $1
