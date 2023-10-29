SELECT EXISTS (SELECT 1 FROM gossip_user WHERE email = $1 AND is_verified = TRUE)
