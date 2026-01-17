# Direct SurrealDB relationship check
# Run this manually in SurrealDB CLI

# Connect to your SurrealDB
surreal sql --conn ws://localhost:7505/rpc --user root --pass root --ns test --db test

# Check what tables exist
INFO FOR DB;

# Check for relationship tables
SELECT * FROM contains LIMIT 5;
SELECT * FROM defines LIMIT 5;

# Check if relationships exist as edges
SELECT id, ->contains->id as contains_targets FROM objects WHERE ->contains LIMIT 5;
SELECT id, <-defines<-id as defined_by FROM objects WHERE <-defines LIMIT 5;

# Show all table names
SELECT name FROM information_schema.tables;

exit
