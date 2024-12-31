export REDIS_ARGS="--daemonize yes" 
./entrypoint.sh

# Wait for Redis to be ready
until redis-cli ping 2>&1 | grep -q PONG; do
  echo "Waiting for Redis..."
  sleep 1
done

# Run initialization commands
echo "Creating indexes..."
redis-cli FT.CREATE idx:orders ON JSON PREFIX 1 order: SCHEMA $.deadline AS deadline NUMERIC $.primary_filler_deadline AS primary_filler_deadline NUMERIC
 
# Stop the background Redis server and start it in the foreground
redis-cli shutdown
export REDIS_ARGS="--daemonize no" 
./entrypoint.sh
