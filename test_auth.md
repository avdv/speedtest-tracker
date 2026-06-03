# Testing API Authentication

## The Issue
You're getting 401 Unauthorized because you need to use the **plaintext token** that was displayed **once** when you created the token, not the hash stored in the database.

## How to Test

1. **Create a new token via the web UI:**
   - Go to `http://localhost:3000/admin/api-tokens`
   - Click "Create New Token"
   - Give it a name and select abilities
   - **COPY THE TOKEN IMMEDIATELY** - it will look like: `kP3mN9xR7...` (40 random characters)

2. **Use the plaintext token in your API request:**
   ```bash
   http localhost:3000/api/v1/ookla/list-servers \
     accept:application/json \
     authorization:"Bearer YOUR_PLAINTEXT_TOKEN_HERE"
   ```

## What's stored in the database?
The database stores SHA-256 hashes of tokens, not the plaintext tokens themselves.  
This is for security - if the database is compromised, the actual tokens aren't exposed.

## How authentication works:
1. You send: `Authorization: Bearer <plaintext_token>`
2. Server hashes your plaintext token with SHA-256
3. Server looks up the hash in the database
4. If found and not expired, authentication succeeds

## Note:
The code now supports both `Bearer` (capital B) and `bearer` (lowercase b) in the header.
