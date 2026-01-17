# Test SurrealDB relationships directly

# Connect to your SurrealDB
surreal sql --conn ws://localhost:7505/rpc --user root --pass root --ns test --db test

# Check if relationships exist
SELECT * FROM contains;
SELECT * FROM defines;

# Check all relationship tables
INFO FOR DB;

# Alternative: Check for any edges
SELECT * FROM (SELECT ->* FROM objects LIMIT 5);
SELECT * FROM (SELECT <-* FROM objects LIMIT 5);

exit
